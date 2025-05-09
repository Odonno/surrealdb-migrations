use ::surrealdb::{
    sql::{
        statements::{
            DefineEventStatement, DefineFieldStatement, DefineStatement, DefineTableStatement,
        },
        Statement,
    },
    Connection, Surreal,
};
use color_eyre::eyre::{eyre, ContextCompat, Result};
use include_dir::Dir;
use itertools::Itertools;
use lexicmp::natural_lexical_cmp;
use sha2::{Digest, Sha256};
use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
};

use crate::{
    common::get_migration_display_name,
    constants::{INITIAL_TRADITIONAL_MIGRATION_FILENAME, SCRIPT_MIGRATION_TABLE_NAME},
    io::{
        self, apply_patch, calculate_definition_using_patches, create_definition_files,
        extract_json_definition_files, filter_except_initial_definition, get_current_definition,
        get_initial_definition, get_migration_definition_diff, SurqlFile,
    },
    models::{ApplyOperation, MigrationDirection, SchemaMigrationDefinition, ScriptMigration},
    surrealdb::{
        self, get_surrealdb_table_definition, is_define_checksum_statement, TransactionAction,
    },
    validate_checksum::{self, ValidateChecksumArgs},
    validate_version_order::{self, ValidateVersionOrderArgs},
};

pub struct ApplyArgs<'a, C: Connection> {
    pub operation: ApplyOperation,
    pub db: &'a Surreal<C>,
    pub dir: Option<&'a Dir<'static>>,
    pub display_logs: bool,
    pub dry_run: bool,
    pub validate_checksum: bool,
    pub validate_version_order: bool,
    pub config_file: Option<&'a Path>,
    pub output: bool,
}

pub async fn main<C: Connection>(args: ApplyArgs<'_, C>) -> Result<()> {
    let ApplyArgs {
        operation,
        db: client,
        dir,
        display_logs,
        dry_run,
        validate_checksum,
        validate_version_order,
        config_file,
        output,
    } = args;

    if validate_version_order {
        let validate_version_order_args = ValidateVersionOrderArgs {
            db: client,
            dir,
            config_file,
        };
        validate_version_order::main(validate_version_order_args).await?;
    }

    if validate_checksum {
        let validate_checksum_args = ValidateChecksumArgs {
            db: client,
            dir,
            config_file,
        };
        validate_checksum::main(validate_checksum_args).await?;
    }

    let display_logs = match dry_run {
        true => false,
        false => display_logs,
    };

    let schemas_files = io::extract_schemas_files(config_file, dir)
        .ok()
        .unwrap_or_default();
    let events_files = io::extract_events_files(config_file, dir)
        .ok()
        .unwrap_or_default();

    let schema_definitions = io::concat_files_content(&schemas_files);
    let event_definitions = io::concat_files_content(&events_files);

    let forward_migrations_files =
        io::extract_migrations_files(config_file, dir, MigrationDirection::Forward);
    let backward_migrations_files =
        io::extract_migrations_files(config_file, dir, MigrationDirection::Backward);

    let use_traditional_approach = schema_definitions.is_empty()
        && event_definitions.is_empty()
        && !forward_migrations_files.is_empty();

    ensures_necessary_files_exists(
        use_traditional_approach,
        &schemas_files,
        &forward_migrations_files,
    )?;

    let use_migration_definitions = !use_traditional_approach;

    const DEFINITIONS_FOLDER: &str = "migrations/definitions";
    let definitions_path = Path::new(DEFINITIONS_FOLDER);

    const INITIAL_DEFINITION_FILENAME: &str = "_initial.json";
    let initial_definition_path = definitions_path.join(INITIAL_DEFINITION_FILENAME);

    if use_migration_definitions {
        if io::can_use_filesystem(config_file)? {
            let should_create_definition_files = match &operation {
                ApplyOperation::Up | ApplyOperation::UpSingle | ApplyOperation::UpTo(_) => true,
                ApplyOperation::Reset | ApplyOperation::DownSingle | ApplyOperation::DownTo(_) => {
                    false
                }
            };

            if should_create_definition_files {
                create_definition_files(
                    config_file,
                    definitions_path.to_path_buf(),
                    initial_definition_path.to_path_buf(),
                    schema_definitions.to_string(),
                    event_definitions.to_string(),
                )?;
            }
        } else if let Some(dir) = dir {
            expect_migration_definitions_to_be_up_to_date(
                schema_definitions.to_string(),
                event_definitions.to_string(),
                dir,
            )?;
        }
    }

    let migrations_applied =
        surrealdb::list_script_migration_ordered_by_execution_date(client).await?;

    let last_migration_applied = migrations_applied.last();

    let migration_files_to_execute = get_migration_files_to_execute(
        forward_migrations_files,
        backward_migrations_files,
        &operation,
        &migrations_applied,
    );

    let migration_direction = MigrationDirection::from(operation);

    match migration_direction {
        MigrationDirection::Forward => {
            apply_migrations(
                config_file,
                definitions_path.to_path_buf(),
                migration_files_to_execute,
                last_migration_applied,
                display_logs,
                client,
                dry_run,
                dir,
                output,
                use_migration_definitions,
            )
            .await?;
        }
        MigrationDirection::Backward => {
            revert_migrations(
                config_file,
                definitions_path.to_path_buf(),
                migration_files_to_execute,
                &migrations_applied,
                display_logs,
                client,
                dry_run,
                dir,
                output,
                use_migration_definitions,
            )
            .await?;
        }
    }

    Ok(())
}

pub fn get_transaction_action(dry_run: bool) -> TransactionAction {
    match dry_run {
        true => TransactionAction::Rollback,
        false => TransactionAction::Commit,
    }
}

fn get_migration_files_to_execute(
    forward_migrations_files: Vec<SurqlFile>,
    backward_migrations_files: Vec<SurqlFile>,
    operation: &ApplyOperation,
    migrations_applied: &[ScriptMigration],
) -> Vec<SurqlFile> {
    let filtered_forward_migrations_files = forward_migrations_files
        .into_iter()
        .filter(|migration_file| {
            filter_migration_file_to_execute(migration_file, operation, migrations_applied, false)
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();

    let filtered_backward_migrations_files = backward_migrations_files
        .into_iter()
        .filter(|migration_file| {
            filter_migration_file_to_execute(migration_file, operation, migrations_applied, true)
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();

    let mut filtered_migrations_files = filtered_forward_migrations_files;
    filtered_migrations_files.extend(filtered_backward_migrations_files);

    let migrations_files = get_sorted_migrations_files(filtered_migrations_files, operation);

    match operation {
        ApplyOperation::UpSingle => migrations_files
            .into_iter()
            .take(migrations_applied.len() + 1)
            .collect::<Vec<_>>(),
        _ => migrations_files,
    }
}

fn get_sorted_migrations_files(
    migrations_files: Vec<SurqlFile>,
    operation: &ApplyOperation,
) -> Vec<SurqlFile> {
    let mut sorted_migrations_files = migrations_files;
    sorted_migrations_files.sort_by(|a, b| match operation {
        ApplyOperation::Up | ApplyOperation::UpSingle | ApplyOperation::UpTo(_) => {
            natural_lexical_cmp(&a.name, &b.name)
        }
        ApplyOperation::Reset | ApplyOperation::DownSingle | ApplyOperation::DownTo(_) => {
            natural_lexical_cmp(&b.name, &a.name)
        }
    });

    sorted_migrations_files
}

fn filter_migration_file_to_execute(
    migration_file: &SurqlFile,
    operation: &ApplyOperation,
    migrations_applied: &[ScriptMigration],
    is_backward_migration: bool,
) -> Result<bool> {
    let migration_direction = MigrationDirection::from(operation);

    match (&migration_direction, is_backward_migration) {
        (MigrationDirection::Forward, true) => return Ok(false),
        (MigrationDirection::Backward, false) => return Ok(false),
        _ => {}
    }

    match &operation {
        ApplyOperation::UpTo(target_migration) => {
            let is_beyond_target =
                natural_lexical_cmp(&migration_file.name, target_migration) == Ordering::Greater;
            if is_beyond_target {
                return Ok(false);
            }
        }
        ApplyOperation::DownTo(target_migration) => {
            let is_target_or_below =
                natural_lexical_cmp(&migration_file.name, target_migration) != Ordering::Greater;
            if is_target_or_below {
                return Ok(false);
            }
        }
        ApplyOperation::DownSingle if migrations_applied.last().is_some() => {
            let target_migration = migrations_applied.last().unwrap();
            let is_below_target =
                natural_lexical_cmp(&migration_file.name, &target_migration.script_name)
                    == Ordering::Less;
            if is_below_target {
                return Ok(false);
            }
        }
        ApplyOperation::Up
        | ApplyOperation::UpSingle
        | ApplyOperation::Reset
        | ApplyOperation::DownSingle => {}
    }

    let has_already_been_applied = migrations_applied
        .iter()
        .any(|migration_applied| migration_applied.script_name == migration_file.name);

    match (&migration_direction, has_already_been_applied) {
        (MigrationDirection::Forward, true) => Ok(false),
        (MigrationDirection::Backward, false) => Ok(false),
        _ => Ok(true),
    }
}

fn expect_migration_definitions_to_be_up_to_date(
    schema_definitions: String,
    event_definitions: String,
    embedded_dir: &Dir<'static>,
) -> Result<()> {
    const DEFINITIONS_FOLDER: &str = "migrations/definitions";
    let definitions_path = Path::new(DEFINITIONS_FOLDER);

    let initial_definition =
        get_initial_definition(None, definitions_path.to_path_buf(), Some(embedded_dir))?;

    let mut definition_files =
        extract_json_definition_files(None, definitions_path, Some(embedded_dir))?;
    definition_files.sort_by(|a, b| natural_lexical_cmp(&a.name, &b.name));
    let definition_files = definition_files;

    let definition_diffs = definition_files
        .into_iter()
        .filter(filter_except_initial_definition)
        .map(|file| file.get_content().unwrap_or_default())
        .collect::<Vec<_>>();

    let last_applied_definition =
        calculate_definition_using_patches(initial_definition, definition_diffs)?;

    let is_up_to_date = schema_definitions == last_applied_definition.schemas
        && event_definitions == last_applied_definition.events;

    if is_up_to_date {
        Ok(())
    } else {
        const ERROR_MESSAGE: &str = "The migration definitions are not up to date. Please run `surrealdb-migrations apply` on your local environment and publish definitions files.";
        Err(eyre!(ERROR_MESSAGE))
    }
}

pub fn ensures_necessary_files_exists(
    use_traditional_approach: bool,
    schemas_files: &[SurqlFile],
    forward_migrations_files: &[SurqlFile],
) -> Result<()> {
    if use_traditional_approach {
        // expect __Initial.surql file (in migrations)
        let has_necessary_files = forward_migrations_files
            .iter()
            .filter(|f| !f.is_down_file())
            .any(|f| f.full_name == INITIAL_TRADITIONAL_MIGRATION_FILENAME);

        if !has_necessary_files {
            return Err(eyre!(
                "The file '{}' should exist.",
                INITIAL_TRADITIONAL_MIGRATION_FILENAME
            ));
        }
    } else {
        // expect script_migration.surql file (in schemas)
        let has_necessary_files = schemas_files
            .iter()
            .filter(|f| !f.is_down_file())
            .any(|f| f.name == SCRIPT_MIGRATION_TABLE_NAME);

        if !has_necessary_files {
            return Err(eyre!(
                "The file '{}' should exist.",
                SCRIPT_MIGRATION_TABLE_NAME
            ));
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn apply_migrations<C: Connection>(
    config_file: Option<&Path>,
    definitions_path: PathBuf,
    migration_files_to_execute: Vec<SurqlFile>,
    last_migration_applied: Option<&ScriptMigration>,
    display_logs: bool,
    client: &Surreal<C>,
    dry_run: bool,
    embedded_dir: Option<&Dir<'static>>,
    output: bool,
    use_migration_definitions: bool,
) -> Result<()> {
    let mut has_applied_schemas = false;
    let mut has_applied_events = false;
    let has_applied_migrations = !&migration_files_to_execute.is_empty();
    let mut current_definition: SchemaMigrationDefinition = Default::default();

    if use_migration_definitions {
        current_definition = match last_migration_applied {
            Some(last_migration_applied) => get_current_definition(
                config_file,
                definitions_path.to_path_buf(),
                last_migration_applied,
                embedded_dir,
            ),
            None => {
                get_initial_definition(config_file, definitions_path.to_path_buf(), embedded_dir)
            }
        }?;

        if !has_applied_migrations {
            let schemas_statements = current_definition.schemas.to_string();
            let events_statements = current_definition.events.to_string();

            if output {
                let query = format!(
                    "{}
        {}",
                    schemas_statements, events_statements,
                );

                println!("-- Initial schema and event definitions --");
                println!("{}", query);
            }

            let schemas_statements = surrealdb::parse_statements(&schemas_statements)?;
            let events_statements = surrealdb::parse_statements(&events_statements)?;

            let statements = schemas_statements
                .into_iter()
                .chain(events_statements.into_iter())
                .collect::<Vec<_>>();

            let transaction_action = get_transaction_action(dry_run);
            surrealdb::apply_in_transaction(client, statements, transaction_action).await?;

            if !current_definition.schemas.is_empty() {
                has_applied_schemas = true;
            }
            if !current_definition.events.is_empty() {
                has_applied_events = true;
            }
        }
    }

    let script_migration_table_definition =
        get_surrealdb_table_definition(client, SCRIPT_MIGRATION_TABLE_NAME).await?;
    let mut supports_checksum = script_migration_table_definition
        .fields
        .contains_key("checksum");

    for migration_file in &migration_files_to_execute {
        let mut schemas_statements = String::new();
        let mut events_statements = String::new();

        if use_migration_definitions {
            let migration_definition_diff = get_migration_definition_diff(
                config_file,
                definitions_path.to_path_buf(),
                migration_file.name.to_string(),
                embedded_dir,
            )?;

            current_definition = match migration_definition_diff {
                Some(migration_definition_diff) => {
                    let schemas = match migration_definition_diff.schemas {
                        Some(schemas_diff) => {
                            apply_patch(current_definition.schemas, schemas_diff)?
                        }
                        None => current_definition.schemas,
                    };
                    let events = match migration_definition_diff.events {
                        Some(events_diff) => apply_patch(current_definition.events, events_diff)?,
                        None => current_definition.events,
                    };

                    SchemaMigrationDefinition { schemas, events }
                }
                None => current_definition,
            };

            schemas_statements = current_definition.schemas.to_string();
            events_statements = current_definition.events.to_string();
        }

        let migration_content = migration_file.get_content().unwrap_or(String::new());

        if output {
            let migration_display_name = get_migration_display_name(&migration_file.name);
            println!("-- Apply migration for {} --", migration_display_name);

            let query = format!(
                "{}
{}
{}
CREATE {} SET script_name = '{}';",
                schemas_statements,
                events_statements,
                migration_content,
                SCRIPT_MIGRATION_TABLE_NAME,
                migration_file.name
            );
            println!("{}", query);
        }

        if display_logs {
            let migration_display_name = get_migration_display_name(&migration_file.name);
            println!("Executing migration {}...", migration_display_name);
        }

        let schemas_statements = surrealdb::parse_statements(&schemas_statements)?;
        let events_statements = surrealdb::parse_statements(&events_statements)?;
        let migration_statements = surrealdb::parse_statements(&migration_content)?;

        supports_checksum = supports_checksum
            || schemas_statements.iter().any(is_define_checksum_statement)
            || migration_statements
                .iter()
                .any(is_define_checksum_statement);

        let mut what = ::surrealdb::sql::Values::default();
        what.0.push(::surrealdb::sql::Value::Table(
            SCRIPT_MIGRATION_TABLE_NAME.into(),
        ));
        let mut set_script_expressions = vec![(
            ::surrealdb::sql::Idiom::from("script_name"),
            ::surrealdb::sql::Operator::Equal,
            ::surrealdb::sql::Value::Strand(migration_file.name.to_string().into()),
        )];
        if supports_checksum {
            let checksum = Sha256::digest(migration_content).to_vec();
            let checksum = hex::encode(checksum);

            set_script_expressions.push((
                ::surrealdb::sql::Idiom::from("checksum"),
                ::surrealdb::sql::Operator::Equal,
                ::surrealdb::sql::Value::Strand(checksum.into()),
            ));
        }
        let mut create_migration_script_statement =
            ::surrealdb::sql::statements::CreateStatement::default();
        create_migration_script_statement.what = what;
        create_migration_script_statement.data = Some(::surrealdb::sql::Data::SetExpression(
            set_script_expressions,
        ));
        create_migration_script_statement.output = Some(::surrealdb::sql::Output::None);

        let statements = schemas_statements
            .into_iter()
            .chain(events_statements.into_iter())
            .chain(migration_statements.into_iter())
            .chain(
                vec![::surrealdb::sql::Statement::Create(
                    create_migration_script_statement,
                )]
                .into_iter(),
            )
            .collect::<Vec<_>>();

        let transaction_action = get_transaction_action(dry_run);
        surrealdb::apply_in_transaction(client, statements, transaction_action).await?;

        if use_migration_definitions {
            if !current_definition.schemas.is_empty() {
                has_applied_schemas = true;
            }
            if !current_definition.events.is_empty() {
                has_applied_events = true;
            }
        }
    }

    if display_logs {
        if has_applied_schemas {
            println!("Schema files successfully executed!");
        }
        if has_applied_events {
            println!("Event files successfully executed!");
        }
        if has_applied_migrations {
            println!("Migration files successfully executed!");
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn revert_migrations<C: Connection>(
    config_file: Option<&Path>,
    definitions_path: PathBuf,
    migration_files_to_execute: Vec<SurqlFile>,
    migrations_applied: &[ScriptMigration],
    display_logs: bool,
    client: &Surreal<C>,
    dry_run: bool,
    embedded_dir: Option<&Dir<'static>>,
    output: bool,
    use_migration_definitions: bool,
) -> Result<()> {
    let last_migration_applied = migrations_applied.last();
    let mut current_definition: SchemaMigrationDefinition = Default::default();

    if use_migration_definitions {
        current_definition = match last_migration_applied {
            Some(last_migration_applied) => get_current_definition(
                config_file,
                definitions_path.to_path_buf(),
                last_migration_applied,
                embedded_dir,
            ),
            None => {
                get_initial_definition(config_file, definitions_path.to_path_buf(), embedded_dir)
            }
        }?;
    }

    let mut has_applied_schemas = false;
    let mut has_applied_events = false;
    let has_applied_migrations = !&migration_files_to_execute.is_empty();

    for migration_file in &migration_files_to_execute {
        // TODO : optimize by getting the range of migration definitions before (avoid recalculation on each migration)
        let migration_reverted = migrations_applied
            .iter()
            .find(|migration_applied| migration_applied.script_name == migration_file.name)
            .context("Migration not found in applied migrations")?;

        let migration_before_reverted = migrations_applied
            .iter()
            .filter(|migration_applied| {
                migration_applied.script_name < migration_reverted.script_name
            })
            .last();

        let mut definition_after_revert: SchemaMigrationDefinition = Default::default();

        if use_migration_definitions {
            definition_after_revert = match migration_before_reverted {
                Some(migration_before_reverted) => get_current_definition(
                    config_file,
                    definitions_path.to_path_buf(),
                    migration_before_reverted,
                    embedded_dir,
                ),
                None => get_initial_definition(
                    config_file,
                    definitions_path.to_path_buf(),
                    embedded_dir,
                ),
            }?;

            if definition_after_revert.schemas != current_definition.schemas {
                has_applied_schemas = true;
            }
            if definition_after_revert.events != current_definition.events {
                has_applied_events = true;
            }
        }

        let migration_statements = migration_file.get_content().unwrap_or(String::new());
        let rollback_schemas_statements = if use_migration_definitions {
            get_rollback_statements(
                &current_definition.schemas,
                &definition_after_revert.schemas,
            )?
        } else {
            vec![]
        };
        let rollback_events_statements = if use_migration_definitions {
            get_rollback_statements(&current_definition.events, &definition_after_revert.events)?
        } else {
            vec![]
        };
        let schemas_statements_after_revert = definition_after_revert.schemas.to_string();
        let events_statements_after_revert = definition_after_revert.events.to_string();

        if output {
            let migration_display_name = get_migration_display_name(&migration_file.name);
            println!("-- Revert migration for {} --", migration_display_name);

            let query = format!(
                "{}
{}
{}
{}
{}
DELETE {} WHERE script_name = '{}';",
                migration_statements,
                rollback_schemas_statements
                    .iter()
                    .map(|s| s.to_string())
                    .join("\n"),
                rollback_events_statements
                    .iter()
                    .map(|s| s.to_string())
                    .join("\n"),
                schemas_statements_after_revert,
                events_statements_after_revert,
                SCRIPT_MIGRATION_TABLE_NAME,
                migration_file.name
            );
            println!("{}", query);
        }

        if display_logs {
            let migration_display_name = get_migration_display_name(&migration_file.name);
            println!("Reverting migration {}...", migration_display_name);
        }

        let migration_statements = surrealdb::parse_statements(&migration_statements)?;
        let schemas_statements_after_revert =
            surrealdb::parse_statements(&schemas_statements_after_revert)?;
        let events_statements_after_revert =
            surrealdb::parse_statements(&events_statements_after_revert)?;

        let mut what = ::surrealdb::sql::Values::default();
        what.0.push(::surrealdb::sql::Value::Table(
            SCRIPT_MIGRATION_TABLE_NAME.into(),
        ));
        let mut cond = ::surrealdb::sql::Cond::default();
        cond.0 =
            ::surrealdb::sql::Value::Expression(Box::new(::surrealdb::sql::Expression::Binary {
                l: ::surrealdb::sql::Value::Idiom("script_name".into()),
                o: ::surrealdb::sql::Operator::Exact,
                r: ::surrealdb::sql::Value::Strand(migration_file.name.to_string().into()),
            }));
        let mut delete_migration_script_statement =
            ::surrealdb::sql::statements::DeleteStatement::default();
        delete_migration_script_statement.what = what;
        delete_migration_script_statement.cond = Some(cond);
        delete_migration_script_statement.output = Some(::surrealdb::sql::Output::None);

        let statements = vec![::surrealdb::sql::Statement::Delete(
            delete_migration_script_statement,
        )]
        .into_iter()
        .chain(migration_statements.into_iter())
        .chain(rollback_schemas_statements.into_iter())
        .chain(rollback_events_statements.into_iter())
        .chain(schemas_statements_after_revert.into_iter())
        .chain(events_statements_after_revert.into_iter())
        .collect::<Vec<_>>();

        let transaction_action = get_transaction_action(dry_run);
        surrealdb::apply_in_transaction(client, statements, transaction_action).await?;

        if use_migration_definitions {
            current_definition = definition_after_revert;
        }
    }

    if display_logs {
        if has_applied_schemas {
            println!("Schema files successfully executed!");
        }
        if has_applied_events {
            println!("Event files successfully executed!");
        }
        if has_applied_migrations {
            println!("Migration files successfully executed!");
        }
    }

    Ok(())
}

fn get_rollback_statements(
    next_statements_str: &str,
    previous_statements_str: &str,
) -> Result<Vec<::surrealdb::sql::Statement>> {
    let next_statements = match next_statements_str.is_empty() {
        true => vec![],
        false => {
            let next_statements = surrealdb::parse_statements(next_statements_str)?;
            next_statements.0 .0
        }
    };

    let previous_statements = match previous_statements_str.is_empty() {
        true => vec![],
        false => {
            let previous_statements = surrealdb::parse_statements(previous_statements_str)?;
            previous_statements.0 .0
        }
    };

    let next_tables_statements = extract_define_table_statements(next_statements.clone());
    let previous_tables_statements = extract_define_table_statements(previous_statements.clone());

    let tables_to_remove = next_tables_statements
        .into_iter()
        .filter(|next_table| {
            previous_tables_statements
                .iter()
                .all(|previous_table| previous_table.name != next_table.name)
        })
        .map(|table| table.name.0)
        .collect::<Vec<_>>();

    let remove_table_statements = tables_to_remove
        .clone()
        .into_iter()
        .map(|table_name| {
            let mut s = ::surrealdb::sql::statements::RemoveTableStatement::default();
            s.name = table_name.into();
            ::surrealdb::sql::Statement::Remove(
                ::surrealdb::sql::statements::RemoveStatement::Table(s),
            )
        })
        .collect::<Vec<_>>();

    let next_fields_statements = extract_define_field_statements(next_statements.clone());
    let previous_fields_statements = extract_define_field_statements(previous_statements.clone());

    let fields_to_remove = next_fields_statements
        .into_iter()
        .filter(|next_field| {
            !previous_fields_statements.iter().any(|previous_field| {
                previous_field.name == next_field.name
                    && previous_field.what.to_string() == next_field.what.to_string()
            })
        })
        .map(|field| (field.name.to_string(), field.what.to_string()))
        .filter(|(_, table_name)| !tables_to_remove.contains(table_name))
        .collect::<Vec<_>>();

    let remove_fields_statements = fields_to_remove
        .into_iter()
        .map(|(field_name, table_name)| {
            let mut s = ::surrealdb::sql::statements::RemoveFieldStatement::default();
            s.name = field_name.into();
            s.what = table_name.into();
            ::surrealdb::sql::Statement::Remove(
                ::surrealdb::sql::statements::RemoveStatement::Field(s),
            )
        })
        .collect::<Vec<_>>();

    let next_events_statements = extract_define_event_statements(next_statements);
    let previous_events_statements = extract_define_event_statements(previous_statements);

    let events_to_remove = next_events_statements
        .into_iter()
        .filter(|next_event| {
            !previous_events_statements.iter().any(|previous_event| {
                previous_event.name == next_event.name
                    && previous_event.what.to_string() == next_event.what.to_string()
            })
        })
        .map(|event| (event.name.to_string(), event.what.to_string()))
        .filter(|(_, table_name)| !tables_to_remove.contains(table_name))
        .collect::<Vec<_>>();

    let remove_events_statements = events_to_remove
        .into_iter()
        .map(|(event_name, table_name)| {
            let mut s = ::surrealdb::sql::statements::RemoveEventStatement::default();
            s.name = event_name.into();
            s.what = table_name.into();
            ::surrealdb::sql::Statement::Remove(
                ::surrealdb::sql::statements::RemoveStatement::Event(s),
            )
        })
        .collect::<Vec<_>>();

    let all_statements = remove_table_statements
        .into_iter()
        .chain(remove_fields_statements)
        .chain(remove_events_statements)
        .collect::<Vec<_>>();

    Ok(all_statements)
}

fn extract_define_table_statements(statements: Vec<Statement>) -> Vec<DefineTableStatement> {
    statements
        .into_iter()
        .filter_map(|statement| match statement {
            Statement::Define(define_statement) => Some(define_statement),
            _ => None,
        })
        .filter_map(|define_statement| match define_statement {
            DefineStatement::Table(define_table_statement) => Some(define_table_statement),
            _ => None,
        })
        .collect::<Vec<_>>()
}

fn extract_define_field_statements(statements: Vec<Statement>) -> Vec<DefineFieldStatement> {
    statements
        .into_iter()
        .filter_map(|statement| match statement {
            Statement::Define(define_statement) => Some(define_statement),
            _ => None,
        })
        .filter_map(|define_statement| match define_statement {
            DefineStatement::Field(define_field_statement) => Some(define_field_statement),
            _ => None,
        })
        .collect::<Vec<_>>()
}

fn extract_define_event_statements(statements: Vec<Statement>) -> Vec<DefineEventStatement> {
    statements
        .into_iter()
        .filter_map(|statement| match statement {
            Statement::Define(define_statement) => Some(define_statement),
            _ => None,
        })
        .filter_map(|define_statement| match define_statement {
            DefineStatement::Event(define_event_statement) => Some(define_event_statement),
            _ => None,
        })
        .collect::<Vec<_>>()
}
