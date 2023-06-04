use std::path::{Path, PathBuf};

use ::surrealdb::{Connection, Surreal};
use anyhow::Result;
use include_dir::Dir;

use crate::{
    common::get_migration_display_name,
    constants::SCRIPT_MIGRATION_TABLE_NAME,
    io::{
        self, apply_patch, create_definition_files, get_current_definition, get_initial_definition,
        get_migration_definition_diff, SurqlFile,
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
    pub config_file: Option<&'a str>,
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

    let migrations_applied =
        surrealdb::list_script_migration_ordered_by_execution_date(client).await?;

    let schemas_files = io::extract_schemas_files(config_file, dir)?;
    let schema_definitions = extract_schema_definitions(schemas_files);

    let events_files = match io::extract_events_files(config_file, dir).ok() {
        Some(files) => files,
        None => vec![],
    };
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
    } else {
        // TODO : Expect last definition (in files) to match the current one
    }

    let last_migration_applied = migrations_applied.last();

    let forward_migrations_files = io::extract_forward_migrations_files(config_file, dir);
    let backward_migrations_files = io::extract_backward_migrations_files(config_file, dir);

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
            )
            .await?;
        }
        MigrationDirection::Backward => {
            revert_migrations(
                migration_files_to_execute,
                schema_definitions.to_string(),
                event_definitions.to_string(),
                display_logs,
                client,
                dry_run,
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
        .map(|file| file.get_content().unwrap_or(String::new()))
        .collect::<Vec<_>>()
        .join("\n")
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

async fn apply_schema_definitions<C: Connection>(
    client: &Surreal<C>,
    schema_definitions: &String,
    dry_run: bool,
) -> Result<()> {
    let action = get_transaction_action(dry_run);
    surrealdb::apply_in_transaction(client, schema_definitions, action).await
}

async fn apply_event_definitions<C: Connection>(
    client: &Surreal<C>,
    event_definitions: &String,
    dry_run: bool,
) -> Result<()> {
    let action = get_transaction_action(dry_run);
    surrealdb::apply_in_transaction(client, event_definitions, action).await
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
        ApplyOperation::UpTo(target_migration) => {
            let is_beyond_target = migration_file.name > *target_migration;
            if is_beyond_target {
                return Ok(false);
            }
        }
        ApplyOperation::Up => {}
        ApplyOperation::Down(target_migration) => {
            let is_target_or_below = migration_file.name <= *target_migration;
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

#[allow(clippy::too_many_arguments)]
async fn apply_migrations<C: Connection>(
    config_file: Option<&str>,
    definitions_path: PathBuf,
    migration_files_to_execute: Vec<SurqlFile>,
    last_migration_applied: Option<&ScriptMigration>,
    display_logs: bool,
    client: &Surreal<C>,
    dry_run: bool,
    embedded_dir: Option<&Dir<'static>>,
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

async fn revert_migrations<C: Connection>(
    migration_files_to_execute: Vec<SurqlFile>,
    definitive_schemas_definition: String,
    definitive_events_definition: String,
    display_logs: bool,
    client: &Surreal<C>,
    dry_run: bool,
) -> Result<()> {
    // TODO : Same logic as apply_migrations

    let has_applied_schemas = !definitive_schemas_definition.is_empty();
    let has_applied_events = !definitive_events_definition.is_empty();

    apply_schema_definitions(client, &definitive_schemas_definition, dry_run).await?;
    apply_event_definitions(client, &definitive_events_definition, dry_run).await?;

    for migration_file in &migration_files_to_execute {
        let inner_query = migration_file.get_content().unwrap_or(String::new());

        let query = format!(
            "{}
DELETE {} WHERE script_name = '{}';",
            inner_query, SCRIPT_MIGRATION_TABLE_NAME, migration_file.name
        );

        if display_logs {
            let migration_display_name = get_migration_display_name(&migration_file.name);
            println!("Reverting migration {}...", migration_display_name);
        }

        let transaction_action = get_transaction_action(dry_run);
        surrealdb::apply_in_transaction(client, &query, transaction_action).await?;
    }

    if display_logs {
        let has_applied_migrations = !&migration_files_to_execute.is_empty();

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
