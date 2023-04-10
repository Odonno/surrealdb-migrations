use fs_extra::dir::{DirEntryAttr, DirEntryValue};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::Path, process};
use surrealdb::{engine::remote::ws::Ws, opt::auth::Root, Surreal};

use crate::{config, definitions};

#[derive(Serialize, Deserialize, Debug)]
struct ScriptMigration {
    script_name: String,
    executed_at: String,
}

fn within_transaction(inner_query: String) -> String {
    format!(
        "BEGIN TRANSACTION;

{}

COMMIT TRANSACTION;",
        inner_query
    )
}

pub async fn main(
    up: Option<String>,
    url: Option<String>,
    ns: Option<String>,
    db: Option<String>,
    username: Option<String>,
    password: Option<String>,
) {
    let db_config = config::retrieve_db_config();

    let url = url.or(db_config.url).unwrap_or("localhost:8000".to_owned());

    let connection = Surreal::new::<Ws>(url.to_owned()).await;

    if let Err(error) = connection {
        eprintln!("{}", error);
        process::exit(1);
    }

    let client = connection.unwrap();

    let username = username.or(db_config.username).unwrap_or("root".to_owned());
    let password = password.or(db_config.password).unwrap_or("root".to_owned());

    client
        .signin(Root {
            username: &username,
            password: &password,
        })
        .await
        .unwrap();

    let ns = ns.or(db_config.ns).unwrap_or("test".to_owned());
    let db = db.or(db_config.db).unwrap_or("test".to_owned());

    client
        .use_ns(ns.to_owned())
        .use_db(db.to_owned())
        .await
        .unwrap();

    let response = client.select("script_migration").await;

    if let Err(error) = response {
        eprintln!("{}", error);
        process::exit(1);
    }

    let mut migrations_applied: Vec<ScriptMigration> = response.unwrap();

    migrations_applied.sort_by_key(|m| m.executed_at.clone());

    let mut config = HashSet::new();
    config.insert(DirEntryAttr::Name);
    config.insert(DirEntryAttr::Path);
    config.insert(DirEntryAttr::IsFile);

    let folder_path = config::retrieve_folder_path();

    const SCHEMAS_FOLDER: &str = "schemas";
    let schemas_path = match folder_path.to_owned() {
        Some(folder_path) => Path::new(&folder_path).join(SCHEMAS_FOLDER),
        None => Path::new(SCHEMAS_FOLDER).to_path_buf(),
    };

    const EVENTS_FOLDER: &str = "events";
    let events_path = match folder_path.to_owned() {
        Some(folder_path) => Path::new(&folder_path).join(EVENTS_FOLDER),
        None => Path::new(EVENTS_FOLDER).to_path_buf(),
    };

    const MIGRATIONS_FOLDER: &str = "migrations";
    let migrations_path = match folder_path.to_owned() {
        Some(folder_path) => Path::new(&folder_path).join(MIGRATIONS_FOLDER),
        None => Path::new(MIGRATIONS_FOLDER).to_path_buf(),
    };

    let schemas_files = fs_extra::dir::ls(schemas_path, &config).unwrap();
    let events_files = fs_extra::dir::ls(events_path, &config).unwrap();
    let migrations_files = fs_extra::dir::ls(migrations_path, &config).unwrap();

    // apply schemas
    let schema_definitions = schemas_files
        .items
        .iter()
        .map(|file| {
            let path = file.get(&DirEntryAttr::Path).unwrap();

            let path = match path {
                DirEntryValue::String(path) => path,
                _ => {
                    eprintln!("Cannot get path to schema files");
                    process::exit(1);
                }
            };

            fs_extra::file::read_to_string(path).unwrap()
        })
        .collect::<Vec<_>>()
        .join("\n");

    let response = client
        .query(within_transaction(schema_definitions.to_owned()))
        .await
        .unwrap();

    let result = response.check();

    if let Err(error) = result {
        eprintln!("{}", error);
        process::exit(1);
    }

    println!("Schema files successfully executed!");

    // apply events
    let event_definitions = events_files
        .items
        .iter()
        .map(|file| {
            let path = file.get(&DirEntryAttr::Path).unwrap();

            let path = match path {
                DirEntryValue::String(path) => path,
                _ => {
                    eprintln!("Cannot get path to event files");
                    process::exit(1);
                }
            };

            fs_extra::file::read_to_string(path).unwrap()
        })
        .collect::<Vec<_>>()
        .join("\n");

    let response = client
        .query(within_transaction(event_definitions.to_owned()))
        .await
        .unwrap();

    let result = response.check();

    if let Err(error) = result {
        eprintln!("{}", error);
        process::exit(1);
    }

    println!("Event files successfully executed!");

    // create definition files
    let last_migration_applied = migrations_applied.last();

    const INITIAL_DEFINITION_FOLDER: &str = "migrations/definitions/_initial.json";
    let initial_definition_path = match folder_path.to_owned() {
        Some(folder_path) => Path::new(&folder_path).join(INITIAL_DEFINITION_FOLDER),
        None => Path::new(INITIAL_DEFINITION_FOLDER).to_path_buf(),
    };

    const DEFINITIONS_FOLDER: &str = "migrations/definitions";
    let definitions_path = match folder_path.to_owned() {
        Some(folder_path) => Path::new(&folder_path).join(DEFINITIONS_FOLDER),
        None => Path::new(DEFINITIONS_FOLDER).to_path_buf(),
    };

    // create folder "migrations/definitions" if not exists
    if !definitions_path.exists() {
        fs_extra::dir::create(&definitions_path, false).unwrap();
    }

    match last_migration_applied {
        Some(last_migration_applied) => {
            let initial_definition =
                fs_extra::file::read_to_string(initial_definition_path).unwrap();
            let initial_definition =
                serde_json::from_str::<definitions::SchemaMigrationDefinition>(&initial_definition)
                    .unwrap();

            // calculate new definition based on all definitions files
            let diff_definition_files = fs_extra::dir::ls(definitions_path, &config).unwrap();

            let definition_diffs = diff_definition_files
                .items
                .iter()
                .filter(|file| {
                    let name = file.get(&DirEntryAttr::Name).unwrap();

                    let name = match name {
                        DirEntryValue::String(name) => name,
                        _ => {
                            eprintln!("Cannot get name to definition files");
                            process::exit(1);
                        }
                    };

                    name != "_initial.json"
                })
                .take_while(|file| match file.get(&DirEntryAttr::Name).unwrap() {
                    DirEntryValue::String(name) => name != &last_migration_applied.script_name,
                    _ => {
                        eprintln!("Cannot get name to definition files");
                        process::exit(1);
                    }
                })
                .map(|file| {
                    let path = file.get(&DirEntryAttr::Path).unwrap();

                    let path = match path {
                        DirEntryValue::String(path) => path,
                        _ => {
                            eprintln!("Cannot get name to definition files");
                            process::exit(1);
                        }
                    };

                    fs_extra::file::read_to_string(path).unwrap()
                })
                .collect::<Vec<_>>();

            let mut last_definition = initial_definition;

            for diff_definition in definition_diffs {
                let definition_diff =
                    serde_json::from_str::<definitions::DefinitionDiff>(&diff_definition).unwrap();

                let schemas = match definition_diff.schemas {
                    Some(schemas_diff) => {
                        let schemas_patch = diffy::Patch::from_str(&schemas_diff).unwrap();
                        diffy::apply(&last_definition.schemas, &schemas_patch).unwrap()
                    }
                    _ => last_definition.schemas,
                };

                let events = match definition_diff.events {
                    Some(events_diff) => {
                        let events_patch = diffy::Patch::from_str(&events_diff).unwrap();
                        diffy::apply(&last_definition.events, &events_patch).unwrap()
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
                    let serialized_definition = serde_json::to_string(&definition_diff).unwrap();
                    fs_extra::file::write_all(&definition_filepath, &serialized_definition)
                        .unwrap();
                }
                false => {
                    // remove definition file if exists
                    let definition_filepath = Path::new(&definition_filepath);

                    if definition_filepath.exists() {
                        fs_extra::file::remove(definition_filepath).unwrap();
                    }
                }
            }
        }
        None => {
            // create folder "migrations/definitions" if not exists
            if !definitions_path.exists() {
                fs_extra::dir::create(&definitions_path, false).unwrap();
            }

            let current_definition = definitions::SchemaMigrationDefinition {
                schemas: schema_definitions,
                events: event_definitions,
            };

            let serialized_definition = serde_json::to_string(&current_definition).unwrap();

            fs_extra::file::write_all(&initial_definition_path, &serialized_definition).unwrap();
        }
    }

    // filter migrations not already applied & apply migrations
    for migration_file in migrations_files.items {
        let name = migration_file.get(&DirEntryAttr::Name).unwrap();
        let path = migration_file.get(&DirEntryAttr::Path).unwrap();
        let is_file = migration_file.get(&DirEntryAttr::IsFile).unwrap();

        let is_file = match is_file {
            DirEntryValue::Boolean(is_file) => is_file,
            _ => {
                eprintln!("Cannot detect if the migration file is a file or a folder");
                process::exit(1);
            }
        };

        if !is_file {
            continue;
        }

        let name = match name {
            DirEntryValue::String(name) => name,
            _ => {
                eprintln!("Cannot get name of the migration file");
                process::exit(1);
            }
        };

        match &up {
            Some(max_migration) => {
                if name > max_migration {
                    continue;
                }
            }
            None => {}
        }

        let has_already_been_applied = migrations_applied
            .iter()
            .any(|migration_applied| &migration_applied.script_name == name);

        if has_already_been_applied {
            continue;
        }

        let path = match path {
            DirEntryValue::String(path) => path,
            _ => {
                eprintln!("Cannot get path to migration files");
                process::exit(1);
            }
        };

        let inner_query = fs_extra::file::read_to_string(path).unwrap();

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

        println!("Executing migration {}...", script_display_name);

        let response = client.query(within_transaction(query)).await.unwrap();

        let result = response.check();

        if let Err(error) = result {
            eprintln!("{}", error);
            process::exit(1);
        }
    }

    println!("Migration files successfully executed!");
}
