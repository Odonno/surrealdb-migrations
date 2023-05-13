use ::surrealdb::{engine::remote::ws::Client, Surreal};
use anyhow::{Context, Result};
use fs_extra::dir::{DirEntryAttr, DirEntryValue, LsResult};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use crate::{
    config,
    constants::{EVENTS_DIR_NAME, MIGRATIONS_DIR_NAME, SCHEMAS_DIR_NAME},
    definitions,
    input::SurrealdbConfiguration,
    models::ScriptMigration,
    surrealdb,
};

pub async fn execute(
    up: Option<String>,
    db_configuration: &SurrealdbConfiguration,
    display_logs: bool,
) -> Result<()> {
    let client = surrealdb::create_surrealdb_client(&db_configuration).await?;

    let migrations_applied =
        surrealdb::list_script_migration_ordered_by_execution_date(&client).await?;

    let mut config = HashSet::new();
    config.insert(DirEntryAttr::Name);
    config.insert(DirEntryAttr::Path);
    config.insert(DirEntryAttr::IsFile);

    let folder_path = config::retrieve_folder_path();

    let schemas_dir_path = concat_path(&folder_path, SCHEMAS_DIR_NAME);
    let events_dir_path = concat_path(&folder_path, EVENTS_DIR_NAME);
    let migrations_dir_path = concat_path(&folder_path, MIGRATIONS_DIR_NAME);

    let schemas_files = fs_extra::dir::ls(schemas_dir_path, &config)?;
    let schema_definitions = extract_schema_definitions(schemas_files);
    apply_schema_definitions(&client, &schema_definitions).await?;

    if display_logs {
        println!("Schema files successfully executed!");
    }

    let event_definitions = if events_dir_path.try_exists()? {
        let events_files = fs_extra::dir::ls(events_dir_path, &config)?;
        let event_definitions = extract_event_definitions(events_files);
        apply_event_definitions(&client, &event_definitions).await?;

        if display_logs {
            println!("Event files successfully executed!");
        }

        event_definitions
    } else {
        String::new()
    };

    let last_migration_applied = migrations_applied.last();

    const INITIAL_DEFINITION_FOLDER: &str = "migrations/definitions/_initial.json";
    let initial_definition_path = concat_path(&folder_path, INITIAL_DEFINITION_FOLDER);

    const DEFINITIONS_FOLDER: &str = "migrations/definitions";
    let definitions_path = concat_path(&folder_path, DEFINITIONS_FOLDER);

    ensures_folder_exists(&definitions_path)?;

    create_definition_files(
        last_migration_applied,
        initial_definition_path,
        definitions_path,
        &config,
        schema_definitions,
        event_definitions,
        folder_path,
    )?;

    let migrations_files = fs_extra::dir::ls(migrations_dir_path, &config)?;
    let migration_files_to_execute =
        get_migration_files_to_execute(&migrations_files, up, &migrations_applied);

    apply_migrations(migration_files_to_execute, display_logs, client).await?;

    if display_logs {
        println!("Migration files successfully executed!");
    }

    Ok(())
}

fn concat_path(folder_path: &Option<String>, dir_name: &str) -> PathBuf {
    match folder_path.to_owned() {
        Some(folder_path) => Path::new(&folder_path).join(dir_name),
        None => Path::new(dir_name).to_path_buf(),
    }
}

fn extract_schema_definitions(schemas_files: LsResult) -> String {
    concat_files_content(schemas_files)
}

fn extract_event_definitions(events_files: LsResult) -> String {
    concat_files_content(events_files)
}

fn concat_files_content(files: LsResult) -> String {
    files
        .items
        .iter()
        .map(|file| map_to_file_content(file).unwrap_or("".to_string())) // TODO : fail when one file fails
        .collect::<Vec<_>>()
        .join("\n")
}

async fn apply_schema_definitions(
    client: &Surreal<Client>,
    schema_definitions: &String,
) -> Result<()> {
    apply_in_transaction(client, schema_definitions).await
}

async fn apply_event_definitions(
    client: &Surreal<Client>,
    event_definitions: &String,
) -> Result<()> {
    apply_in_transaction(client, event_definitions).await
}

async fn apply_in_transaction(client: &Surreal<Client>, inner_query: &String) -> Result<()> {
    let response = client
        .query(surrealdb::within_transaction(inner_query.to_owned()))
        .await?;

    response.check()?;
    Ok(())
}

fn ensures_folder_exists(dir_path: &PathBuf) -> Result<()> {
    if !dir_path.exists() {
        fs_extra::dir::create_all(&dir_path, false)?;
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
    folder_path: Option<String>,
) -> Result<()> {
    match last_migration_applied {
        Some(last_migration_applied) => {
            let initial_definition = fs_extra::file::read_to_string(initial_definition_path)?;
            let initial_definition = serde_json::from_str::<definitions::SchemaMigrationDefinition>(
                &initial_definition,
            )?;

            // calculate new definition based on all definitions files
            let diff_definition_files = fs_extra::dir::ls(definitions_path, &config)?;

            let definition_diffs = diff_definition_files
                .items
                .iter()
                .filter(|file| filter_expect_initial_definition(file).unwrap_or(false))
                .take_while(|file| {
                    take_while_not_applied(file, last_migration_applied).unwrap_or(false)
                })
                .map(|file| map_to_file_content(file))
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
            let definition_filepath = match folder_path.to_owned() {
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
    migrations_files: &'a LsResult,
    up: Option<String>,
    migrations_applied: &'a Vec<ScriptMigration>,
) -> Vec<&'a HashMap<DirEntryAttr, DirEntryValue>> {
    get_sorted_migrations_files(&migrations_files)
        .into_iter()
        .filter(|migration_file| {
            filter_migration_file_to_execute(migration_file, up.to_owned(), &migrations_applied)
                .unwrap_or(false)
        })
        .collect::<Vec<_>>()
}

fn get_sorted_migrations_files(
    migrations_files: &LsResult,
) -> Vec<&HashMap<DirEntryAttr, DirEntryValue>> {
    let mut sorted_migrations_files = migrations_files.items.iter().collect::<Vec<_>>();
    sorted_migrations_files.sort_by(|a, b| {
        let a = a.get(&DirEntryAttr::Name);
        let b = b.get(&DirEntryAttr::Name);

        let a = match a {
            Some(DirEntryValue::String(a)) => Some(a),
            _ => None,
        };

        let b = match b {
            Some(DirEntryValue::String(b)) => Some(b),
            _ => None,
        };

        a.cmp(&b)
    });

    sorted_migrations_files
}

fn filter_migration_file_to_execute(
    migration_file: &&std::collections::HashMap<DirEntryAttr, DirEntryValue>,
    up: Option<String>,
    migrations_applied: &Vec<ScriptMigration>,
) -> Result<bool> {
    let is_file = migration_file
        .get(&DirEntryAttr::IsFile)
        .context("Cannot detect if the migration file is a file or a folder")?;
    let is_file = match is_file {
        DirEntryValue::Boolean(is_file) => Some(is_file),
        _ => None,
    };
    let is_file = is_file.context("Cannot detect if the migration file is a file or a folder")?;

    if !is_file {
        return Ok(false);
    }

    let name = migration_file
        .get(&DirEntryAttr::Name)
        .context("Cannot get name of the migration file")?;
    let name = match name {
        DirEntryValue::String(name) => Some(name),
        _ => None,
    };
    let name = name.context("Cannot get name of the migration file")?;

    match &up {
        Some(max_migration) => {
            if name > max_migration {
                return Ok(false);
            }
        }
        None => {}
    }

    let has_already_been_applied = migrations_applied
        .iter()
        .any(|migration_applied| &migration_applied.script_name == name);

    if has_already_been_applied {
        return Ok(false);
    }

    return Ok(true);
}

async fn apply_migrations(
    migration_files_to_execute: Vec<&HashMap<DirEntryAttr, DirEntryValue>>,
    display_logs: bool,
    client: Surreal<Client>,
) -> Result<()> {
    for migration_file in migration_files_to_execute {
        let name = migration_file
            .get(&DirEntryAttr::Name)
            .context("Cannot get name of the migration file")?;
        let name: Option<&String> = match name {
            DirEntryValue::String(name) => Some(name),
            _ => None,
        };
        let name = name.context("Cannot get name of the migration file")?;

        let path = migration_file
            .get(&DirEntryAttr::Path)
            .context("Cannot get path of the migration file")?;
        let path = match path {
            DirEntryValue::String(path) => Some(path),
            _ => None,
        };
        let path = path.context("Cannot get path of the migration file")?;

        let inner_query = fs_extra::file::read_to_string(path)?;

        let query = format!(
            "{}
CREATE script_migration SET script_name = '{}';",
            inner_query, name
        );

        let script_display_name = name
            .split("_")
            .skip(2)
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join("_");

        if display_logs {
            println!("Executing migration {}...", script_display_name);
        }

        apply_in_transaction(&client, &query).await?;
    }

    Ok(())
}
