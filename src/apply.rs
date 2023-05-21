use ::surrealdb::{engine::any::Any, Surreal};
use anyhow::{Context, Result};
use fs_extra::dir::{DirEntryAttr, DirEntryValue};
use include_dir::Dir;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use crate::{
    config,
    constants::{
        DOWN_MIGRATIONS_DIR_NAME, EVENTS_DIR_NAME, MIGRATIONS_DIR_NAME, SCHEMAS_DIR_NAME,
        SCRIPT_MIGRATION_TABLE_NAME,
    },
    definitions,
    io::{self, SurqlFile},
    models::ScriptMigration,
    surrealdb::{self, TransactionAction},
};

pub struct ApplyArgs<'a> {
    pub operation: ApplyOperation,
    pub db: &'a Surreal<Any>,
    pub dir: Option<&'a Dir<'a>>,
    pub display_logs: bool,
    pub dry_run: bool,
}

pub enum ApplyOperation {
    Up,
    UpTo(String),
    Down(String),
}

pub async fn main(args: ApplyArgs<'_>) -> Result<()> {
    let ApplyArgs {
        operation,
        db: client,
        dir,
        display_logs,
        dry_run,
    } = args;

    let display_logs = match dry_run {
        true => false,
        false => display_logs,
    };

    let migrations_applied =
        surrealdb::list_script_migration_ordered_by_execution_date(client).await?;

    let mut config = HashSet::new();
    config.insert(DirEntryAttr::Name);
    config.insert(DirEntryAttr::Path);
    config.insert(DirEntryAttr::IsFile);
    config.insert(DirEntryAttr::FullName); // Used to filter migrations files (from down files)

    let folder_path = config::retrieve_folder_path();

    let schemas_files = io::extract_surql_files(Path::new(SCHEMAS_DIR_NAME).to_path_buf(), dir)?;

    let schema_definitions = extract_schema_definitions(schemas_files);
    apply_schema_definitions(client, &schema_definitions, dry_run).await?;

    if display_logs {
        println!("Schema files successfully executed!");
    }

    let events_dir = Path::new(EVENTS_DIR_NAME).to_path_buf();
    let events_files = match io::extract_surql_files(events_dir, dir).ok() {
        Some(files) => files,
        None => vec![],
    };

    let event_definitions = if events_files.len() > 0 {
        let event_definitions = extract_event_definitions(events_files);
        apply_event_definitions(client, &event_definitions, dry_run).await?;

        if display_logs {
            println!("Event files successfully executed!");
        }

        event_definitions
    } else {
        String::new()
    };

    if io::can_use_filesystem() {
        const DEFINITIONS_FOLDER: &str = "migrations/definitions";
        let definitions_path = io::concat_path(&folder_path, DEFINITIONS_FOLDER);

        const INITIAL_DEFINITION_FILENAME: &str = "_initial.json";
        let initial_definition_path = definitions_path.join(INITIAL_DEFINITION_FILENAME);

        ensures_folder_exists(&definitions_path)?;

        let should_create_definition_files = match &operation {
            ApplyOperation::Up => true,
            ApplyOperation::UpTo(_) => true,
            ApplyOperation::Down(_) => false,
        };

        if should_create_definition_files {
            let last_migration_applied = migrations_applied.last();

            create_definition_files(
                last_migration_applied,
                initial_definition_path,
                definitions_path,
                &config,
                schema_definitions,
                event_definitions,
                &folder_path,
            )?;
        }
    }

    let migrations_dir = Path::new(MIGRATIONS_DIR_NAME).to_path_buf();
    let migrations_files = match io::extract_surql_files(migrations_dir, dir).ok() {
        Some(files) => files,
        None => vec![],
    };

    let down_migrations_dir = Path::new(MIGRATIONS_DIR_NAME).join(DOWN_MIGRATIONS_DIR_NAME);
    let down_migrations_files = match io::extract_surql_files(down_migrations_dir, dir).ok() {
        Some(files) => files,
        None => vec![],
    };

    let migration_files_to_execute = get_migration_files_to_execute(
        migrations_files,
        down_migrations_files,
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
            apply_migrations(migration_files_to_execute, display_logs, client, dry_run).await?;
        }
        MigrationDirection::Backward => {
            revert_migrations(migration_files_to_execute, display_logs, client, dry_run).await?;
        }
    }

    if display_logs {
        println!("Migration files successfully executed!");
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
    files
        .iter()
        .map(|file| file.content.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

async fn apply_schema_definitions(
    client: &Surreal<Any>,
    schema_definitions: &String,
    dry_run: bool,
) -> Result<()> {
    let action = get_transaction_action(dry_run);
    surrealdb::apply_in_transaction(client, schema_definitions, action).await
}

async fn apply_event_definitions(
    client: &Surreal<Any>,
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

fn ensures_folder_exists(dir_path: &PathBuf) -> Result<()> {
    if !dir_path.exists() {
        fs_extra::dir::create_all(dir_path, false)?;
    }

    Ok(())
}

fn filter_expect_initial_definition(file: &&HashMap<DirEntryAttr, DirEntryValue>) -> Result<bool> {
    let name = file
        .get(&DirEntryAttr::Name)
        .context("Cannot get name to definition files")?;

    let name = match name {
        DirEntryValue::String(name) => Some(name.to_owned()),
        _ => None,
    };

    let result = name != Some("_initial.json".to_string());
    Ok(result)
}

fn take_while_not_applied(
    file: &&HashMap<DirEntryAttr, DirEntryValue>,
    last_migration_applied: &ScriptMigration,
) -> Result<bool> {
    let name = file
        .get(&DirEntryAttr::Name)
        .context("Cannot get name to definition files")?;
    let name = match name {
        DirEntryValue::String(name) => Some(name),
        _ => None,
    };
    let name = name.context("Cannot get name to definition files")?;

    let result = name != &last_migration_applied.script_name;
    Ok(result)
}

fn map_to_file_content(file: &HashMap<DirEntryAttr, DirEntryValue>) -> Result<String> {
    let path = file
        .get(&DirEntryAttr::Path)
        .context("Cannot get path to definition files")?;

    let path = match path {
        DirEntryValue::String(path) => Some(path),
        _ => None,
    };
    let path = path.context("Cannot get path to definition files")?;

    fs_extra::file::read_to_string(path).context("Cannot get path to definition files")
}

fn create_definition_files(
    last_migration_applied: Option<&ScriptMigration>,
    initial_definition_path: PathBuf,
    definitions_path: PathBuf,
    config: &HashSet<DirEntryAttr>,
    schema_definitions: String,
    event_definitions: String,
    folder_path: &Option<String>,
) -> Result<()> {
    match last_migration_applied {
        Some(last_migration_applied) => {
            let initial_definition = fs_extra::file::read_to_string(initial_definition_path)?;
            let initial_definition = serde_json::from_str::<definitions::SchemaMigrationDefinition>(
                &initial_definition,
            )?;

            // calculate new definition based on all definitions files
            let diff_definition_files = fs_extra::dir::ls(definitions_path, config)?;

            let definition_diffs = diff_definition_files
                .items
                .iter()
                .filter(|file| filter_expect_initial_definition(file).unwrap_or(false))
                .take_while(|file| {
                    take_while_not_applied(file, last_migration_applied).unwrap_or(false)
                })
                .map(map_to_file_content)
                .collect::<Vec<_>>();

            let mut last_definition = initial_definition;

            for diff_definition in definition_diffs {
                let diff_definition = diff_definition?;

                let definition_diff =
                    serde_json::from_str::<definitions::DefinitionDiff>(&diff_definition)?;

                let schemas = match definition_diff.schemas {
                    Some(schemas_diff) => {
                        let schemas_patch = diffy::Patch::from_str(&schemas_diff)?;
                        diffy::apply(&last_definition.schemas, &schemas_patch)?
                    }
                    _ => last_definition.schemas,
                };

                let events = match definition_diff.events {
                    Some(events_diff) => {
                        let events_patch = diffy::Patch::from_str(&events_diff)?;
                        diffy::apply(&last_definition.events, &events_patch)?
                    }
                    _ => last_definition.events,
                };

                last_definition = definitions::SchemaMigrationDefinition { schemas, events };
            }

            // make diff between new definition and last definition
            let current_definition = definitions::SchemaMigrationDefinition {
                schemas: schema_definitions,
                events: event_definitions,
            };

            // save definition if any changes
            let definition_filepath = format!(
                "migrations/definitions/{}.json",
                last_migration_applied.script_name
            );
            let definition_filepath = match folder_path {
                Some(folder_path) => Path::new(&folder_path).join(definition_filepath),
                None => Path::new(&definition_filepath).to_path_buf(),
            };

            let has_schema_diffs =
                last_definition.schemas.trim() != current_definition.schemas.trim();
            let has_event_diffs = last_definition.events.trim() != current_definition.events.trim();

            let schemas_diffs = match has_schema_diffs {
                true => Some(
                    diffy::create_patch(&last_definition.schemas, &current_definition.schemas)
                        .to_string(),
                ),
                false => None,
            };

            let events_diffs = match has_event_diffs {
                true => Some(
                    diffy::create_patch(&last_definition.events, &current_definition.events)
                        .to_string(),
                ),
                false => None,
            };

            let definition_diff = definitions::DefinitionDiff {
                schemas: schemas_diffs,
                events: events_diffs,
            };

            // create definition file if any changes
            let has_changes = definition_diff.schemas.is_some() || definition_diff.events.is_some();

            match has_changes {
                true => {
                    let serialized_definition = serde_json::to_string(&definition_diff)?;
                    fs_extra::file::write_all(&definition_filepath, &serialized_definition)?;
                }
                false => {
                    // remove definition file if exists
                    let definition_filepath = Path::new(&definition_filepath);

                    if definition_filepath.exists() {
                        fs_extra::file::remove(definition_filepath)?;
                    }
                }
            }
        }
        None => {
            // create folder "migrations/definitions" if not exists
            if !definitions_path.exists() {
                fs_extra::dir::create(&definitions_path, false)?;
            }

            let current_definition = definitions::SchemaMigrationDefinition {
                schemas: schema_definitions,
                events: event_definitions,
            };

            let serialized_definition = serde_json::to_string(&current_definition)?;

            fs_extra::file::write_all(&initial_definition_path, &serialized_definition)?;
        }
    }

    Ok(())
}

fn get_migration_files_to_execute<'a>(
    migrations_files: Vec<SurqlFile>,
    down_migrations_files: Vec<SurqlFile>,
    operation: &ApplyOperation,
    migrations_applied: &'a Vec<ScriptMigration>,
) -> Vec<SurqlFile> {
    let mut filtered_migrations_files = migrations_files
        .into_iter()
        .filter(|migration_file| {
            filter_migration_file_to_execute(migration_file, operation, migrations_applied, false)
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();

    let filtered_down_migrations_files = down_migrations_files
        .into_iter()
        .filter(|migration_file| {
            filter_migration_file_to_execute(migration_file, &operation, &migrations_applied, true)
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();

    filtered_migrations_files.extend(filtered_down_migrations_files);

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
    is_from_down_folder: bool,
) -> Result<bool> {
    let is_down_file = match is_from_down_folder {
        true => true,
        false => {
            let is_down_file = migration_file.full_name.ends_with(".down.surql");
            is_down_file
        }
    };

    let migration_direction = match &operation {
        ApplyOperation::Up => MigrationDirection::Forward,
        ApplyOperation::UpTo(_) => MigrationDirection::Forward,
        ApplyOperation::Down(_) => MigrationDirection::Backward,
    };

    match (&migration_direction, is_down_file) {
        (MigrationDirection::Forward, true) => return Ok(false),
        (MigrationDirection::Backward, false) => return Ok(false),
        _ => {}
    }

    match &operation {
        ApplyOperation::UpTo(target_migration) => {
            let is_beyond_target = migration_file.name > target_migration.to_string();
            if is_beyond_target {
                return Ok(false);
            }
        }
        ApplyOperation::Up => {}
        ApplyOperation::Down(target_migration) => {
            let is_target_or_below = migration_file.name <= target_migration.to_string();
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

async fn apply_migrations(
    migration_files_to_execute: Vec<SurqlFile>,
    display_logs: bool,
    client: &Surreal<Any>,
    dry_run: bool,
) -> Result<()> {
    for migration_file in migration_files_to_execute {
        let inner_query = migration_file.content;

        let query = format!(
            "{}
CREATE {} SET script_name = '{}';",
            inner_query, SCRIPT_MIGRATION_TABLE_NAME, migration_file.name
        );

        let script_display_name = migration_file
            .name
            .split("_")
            .skip(2)
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join("_");

        if display_logs {
            println!("Executing migration {}...", script_display_name);
        }

        let transaction_action = get_transaction_action(dry_run);
        surrealdb::apply_in_transaction(client, &query, transaction_action).await?;
    }

    Ok(())
}

async fn revert_migrations(
    migration_files_to_execute: Vec<SurqlFile>,
    display_logs: bool,
    client: &Surreal<Any>,
    dry_run: bool,
) -> Result<()> {
    for migration_file in migration_files_to_execute {
        let inner_query = migration_file.content;

        let query = format!(
            "{}
DELETE {} WHERE script_name = '{}';",
            inner_query, SCRIPT_MIGRATION_TABLE_NAME, migration_file.name
        );

        let script_display_name = migration_file
            .name
            .split("_")
            .skip(2)
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join("_");

        if display_logs {
            println!("Reverting migration {}...", script_display_name);
        }

        let transaction_action = get_transaction_action(dry_run);
        surrealdb::apply_in_transaction(client, &query, transaction_action).await?;
    }

    Ok(())
}
