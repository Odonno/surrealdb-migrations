use std::path::{Path, PathBuf};

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

use crate::{
    common::get_migration_display_name,
    constants::SCRIPT_MIGRATION_TABLE_NAME,
    io::{
        self, apply_patch, calculate_definition_using_patches, create_definition_files,
        extract_json_definition_files, filter_except_initial_definition, get_current_definition,
        get_initial_definition, get_migration_definition_diff, SurqlFile,
    },
    models::{SchemaMigrationDefinition, ScriptMigration},
    surrealdb::{self, TransactionAction},
    validate_version_order::{self, ValidateVersionOrderArgs},
};

pub struct ApplyArgs<'a, C: Connection> {
    pub operation: ApplyOperation,
    pub db: &'a Surreal<C>,
    pub dir: Option<&'a Dir<'static>>,
    pub display_logs: bool,
    pub dry_run: bool,
    pub validate_version_order: bool,
    pub config_file: Option<&'a Path>,
    pub output: bool,
}

pub enum ApplyOperation {
    Up,
    UpTo(String),
    Down(String),
}

pub async fn main<C: Connection>(args: ApplyArgs<'_, C>) -> Result<()> {
    let ApplyArgs {
        operation,
        db: client,
        dir,
        display_logs,
        dry_run,
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

    let display_logs = match dry_run {
        true => false,
        false => display_logs,
    };

    let schemas_files = io::extract_schemas_files(config_file, dir)?;
    let schema_definitions = extract_schema_definitions(schemas_files);

    let events_files = io::extract_events_files(config_file, dir)
        .ok()
        .unwrap_or_default();
    let event_definitions = extract_event_definitions(events_files);

    const DEFINITIONS_FOLDER: &str = "migrations/definitions";
    let definitions_path = Path::new(DEFINITIONS_FOLDER);

    const INITIAL_DEFINITION_FILENAME: &str = "_initial.json";
    let initial_definition_path = definitions_path.join(INITIAL_DEFINITION_FILENAME);

    if io::can_use_filesystem(config_file)? {
        let should_create_definition_files = match &operation {
            ApplyOperation::Up => true,
            ApplyOperation::UpTo(_) => true,
            ApplyOperation::Down(_) => false,
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

    let forward_migrations_files = io::extract_forward_migrations_files(config_file, dir);
    let backward_migrations_files = io::extract_backward_migrations_files(config_file, dir);

    let migrations_applied =
        surrealdb::list_script_migration_ordered_by_execution_date(client).await?;

    let last_migration_applied = migrations_applied.last();

    let migration_files_to_execute = get_migration_files_to_execute(
        forward_migrations_files,
        backward_migrations_files,
        &operation,
        &migrations_applied,
    );

    let migration_direction = match &operation {
        ApplyOperation::Up => MigrationDirection::Forward,
        ApplyOperation::UpTo(_) => MigrationDirection::Forward,
        ApplyOperation::Down(_) => MigrationDirection::Backward,
    };

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
            )
            .await?;
        }
    }

    Ok(())
}

enum MigrationDirection {
    Forward,
    Backward,
}

fn extract_schema_definitions(schemas_files: Vec<SurqlFile>) -> String {
    concat_files_content(schemas_files)
}

fn extract_event_definitions(events_files: Vec<SurqlFile>) -> String {
    concat_files_content(events_files)
}

fn concat_files_content(files: Vec<SurqlFile>) -> String {
    let mut ordered_files = files;
    ordered_files.sort_by(|a, b| a.name.cmp(&b.name));

    ordered_files
        .iter()
        .map(|file| file.get_content().unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n")
}

fn get_transaction_action(dry_run: bool) -> TransactionAction {
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

    get_sorted_migrations_files(filtered_migrations_files, operation)
}

fn get_sorted_migrations_files(
    migrations_files: Vec<SurqlFile>,
    operation: &ApplyOperation,
) -> Vec<SurqlFile> {
    let mut sorted_migrations_files = migrations_files;
    sorted_migrations_files.sort_by(|a, b| match operation {
        ApplyOperation::Up => a.name.cmp(&b.name),
        ApplyOperation::UpTo(_) => a.name.cmp(&b.name),
        ApplyOperation::Down(_) => b.name.cmp(&a.name),
    });

    sorted_migrations_files
}

fn filter_migration_file_to_execute(
    migration_file: &SurqlFile,
    operation: &ApplyOperation,
    migrations_applied: &[ScriptMigration],
    is_backward_migration: bool,
) -> Result<bool> {
    let migration_direction = match &operation {
        ApplyOperation::Up => MigrationDirection::Forward,
        ApplyOperation::UpTo(_) => MigrationDirection::Forward,
        ApplyOperation::Down(_) => MigrationDirection::Backward,
    };

    match (&migration_direction, is_backward_migration) {
        (MigrationDirection::Forward, true) => return Ok(false),
        (MigrationDirection::Backward, false) => return Ok(false),
        _ => {}
    }

    match &operation {
        ApplyOperation::Up => {}
        ApplyOperation::UpTo(target_migration) => {
            let is_beyond_target = &migration_file.name > target_migration;
            if is_beyond_target {
                return Ok(false);
            }
        }
        ApplyOperation::Down(target_migration) => {
            let is_target_or_below = &migration_file.name <= target_migration;
            if is_target_or_below {
                return Ok(false);
            }
        }
    }

    let has_already_been_applied = migrations_applied
        .iter()
        .any(|migration_applied| migration_applied.script_name == migration_file.name);

    match (&migration_direction, has_already_been_applied) {
        (MigrationDirection::Forward, true) => return Ok(false),
        (MigrationDirection::Backward, false) => return Ok(false),
        _ => {}
    }

    Ok(true)
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
    definition_files.sort_by(|a, b| a.name.cmp(&b.name));
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
) -> Result<()> {
    let mut has_applied_schemas = false;
    let mut has_applied_events = false;

    let mut current_definition = match last_migration_applied {
        Some(last_migration_applied) => get_current_definition(
            config_file,
            definitions_path.to_path_buf(),
            last_migration_applied,
            embedded_dir,
        ),
        None => get_initial_definition(config_file, definitions_path.to_path_buf(), embedded_dir),
    }?;

    let has_applied_migrations = !&migration_files_to_execute.is_empty();

    if !has_applied_migrations {
        let schemas_statements = current_definition.schemas.to_string();
        let events_statements = current_definition.events.to_string();

        let query = format!(
            "{}
{}",
            schemas_statements, events_statements,
        );

        if output {
            println!("-- Initial schema and event definitions --");
            println!("{}", query);
        }

        let transaction_action = get_transaction_action(dry_run);
        surrealdb::apply_in_transaction(client, &query, transaction_action).await?;

        if !current_definition.schemas.is_empty() {
            has_applied_schemas = true;
        }
        if !current_definition.events.is_empty() {
            has_applied_events = true;
        }
    }

    for migration_file in &migration_files_to_execute {
        let migration_definition_diff = get_migration_definition_diff(
            config_file,
            definitions_path.to_path_buf(),
            migration_file.name.to_string(),
            embedded_dir,
        )?;

        current_definition = match migration_definition_diff {
            Some(migration_definition_diff) => {
                let schemas = match migration_definition_diff.schemas {
                    Some(schemas_diff) => apply_patch(current_definition.schemas, schemas_diff)?,
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

        let schemas_statements = current_definition.schemas.to_string();
        let events_statements = current_definition.events.to_string();
        let migration_statements = migration_file.get_content().unwrap_or(String::new());

        let query = format!(
            "{}
{}
{}
CREATE {} SET script_name = '{}';",
            schemas_statements,
            events_statements,
            migration_statements,
            SCRIPT_MIGRATION_TABLE_NAME,
            migration_file.name
        );

        if output {
            let migration_display_name = get_migration_display_name(&migration_file.name);
            println!("-- Apply migration for {} --", migration_display_name);
            println!("{}", query);
        }

        if display_logs {
            let migration_display_name = get_migration_display_name(&migration_file.name);
            println!("Executing migration {}...", migration_display_name);
        }

        let transaction_action = get_transaction_action(dry_run);
        surrealdb::apply_in_transaction(client, &query, transaction_action).await?;

        if !current_definition.schemas.is_empty() {
            has_applied_schemas = true;
        }
        if !current_definition.events.is_empty() {
            has_applied_events = true;
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
) -> Result<()> {
    let last_migration_applied = migrations_applied.last();

    let mut current_definition = match last_migration_applied {
        Some(last_migration_applied) => get_current_definition(
            config_file,
            definitions_path.to_path_buf(),
            last_migration_applied,
            embedded_dir,
        ),
        None => get_initial_definition(config_file, definitions_path.to_path_buf(), embedded_dir),
    }?;

    let has_applied_migrations = !&migration_files_to_execute.is_empty();

    let mut has_applied_schemas = false;
    let mut has_applied_events = false;

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

        let definition_after_revert = match migration_before_reverted {
            Some(migration_before_reverted) => get_current_definition(
                config_file,
                definitions_path.to_path_buf(),
                migration_before_reverted,
                embedded_dir,
            ),
            None => {
                get_initial_definition(config_file, definitions_path.to_path_buf(), embedded_dir)
            }
        }?;

        if definition_after_revert.schemas != current_definition.schemas {
            has_applied_schemas = true;
        }
        if definition_after_revert.events != current_definition.events {
            has_applied_events = true;
        }

        let migration_statements = migration_file.get_content().unwrap_or(String::new());
        let rollback_schemas_statements = get_rollback_statements(
            &current_definition.schemas,
            &definition_after_revert.schemas,
        )?;
        let rollback_events_statements =
            get_rollback_statements(&current_definition.events, &definition_after_revert.events)?;
        let schemas_statements_after_revert = definition_after_revert.schemas.to_string();
        let events_statements_after_revert = definition_after_revert.events.to_string();

        let query = format!(
            "{}
{}
{}
{}
{}
DELETE {} WHERE script_name = '{}';",
            migration_statements,
            rollback_schemas_statements,
            rollback_events_statements,
            schemas_statements_after_revert,
            events_statements_after_revert,
            SCRIPT_MIGRATION_TABLE_NAME,
            migration_file.name
        );

        if output {
            let migration_display_name = get_migration_display_name(&migration_file.name);
            println!("-- Revert migration for {} --", migration_display_name);
            println!("{}", query);
        }

        if display_logs {
            let migration_display_name = get_migration_display_name(&migration_file.name);
            println!("Reverting migration {}...", migration_display_name);
        }

        let transaction_action = get_transaction_action(dry_run);
        surrealdb::apply_in_transaction(client, &query, transaction_action).await?;

        current_definition = definition_after_revert;
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
) -> Result<String> {
    let next_statements = match next_statements_str.is_empty() {
        true => vec![],
        false => {
            let next_statements = ::surrealdb::sql::parse(next_statements_str)?;
            next_statements.0 .0
        }
    };

    let previous_statements = match previous_statements_str.is_empty() {
        true => vec![],
        false => {
            let previous_statements = ::surrealdb::sql::parse(previous_statements_str)?;
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
        .map(|table_name| format!("REMOVE TABLE {};", table_name))
        .collect::<Vec<_>>()
        .join("\n");

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
        .map(|(field_name, table_name)| format!("REMOVE FIELD {} ON {};", field_name, table_name))
        .collect::<Vec<_>>()
        .join("\n");

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
        .map(|(event_name, table_name)| format!("REMOVE EVENT {} ON {};", event_name, table_name))
        .collect::<Vec<_>>()
        .join("\n");

    let all_statements = [
        remove_table_statements,
        remove_fields_statements,
        remove_events_statements,
    ];
    let all_statements = all_statements.join("\n");

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

#[cfg(test)]
mod tests {
    use crate::io::create_surql_file;

    use super::*;

    #[test]
    fn concat_empty_list_of_files() {
        let result = concat_files_content(vec![]);
        assert_eq!(result, "");
    }

    #[test]
    fn concat_files_in_alphabetic_order() {
        let files = vec![
            create_surql_file("a.text", "Text of a file"),
            create_surql_file("c.text", "Text of c file"),
            create_surql_file("b.text", "Text of b file"),
        ];

        let result = concat_files_content(files);
        assert_eq!(result, "Text of a file\nText of b file\nText of c file");
    }
}
