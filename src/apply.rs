use std::{collections::HashSet, path::Path};

use fs_extra::dir::{DirEntryAttr, DirEntryValue};
use reqwest::{
    header::{HeaderMap, ACCEPT},
    Response,
};
use serde::{Deserialize, Serialize};

use crate::definitions;

#[derive(Serialize, Deserialize, Debug)]
struct ScriptMigration {
    id: String,
    script_name: String,
    executed_at: String,
}

struct SurrealDbQueryParams {
    url: String,
    ns: String,
    db: String,
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct EmptySurrealDbInstructionResponse {
    time: String,
    status: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct SurrealDbInstructionResponse<T> {
    time: String,
    status: String,
    result: Option<Vec<T>>,
}

type EmptySurrealDbResponse = Vec<EmptySurrealDbInstructionResponse>;
type SurrealDbResponse<T> = Vec<SurrealDbInstructionResponse<T>>;

type GetScriptMigrationsResponse = SurrealDbResponse<ScriptMigration>;

type ExecuteSchemaResponse = EmptySurrealDbResponse;
type ExecuteEventResponse = EmptySurrealDbResponse;
type ExecuteMigrationResponse = EmptySurrealDbResponse;

async fn execute_query(params: &SurrealDbQueryParams, query: String) -> Response {
    let client = reqwest::Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, "application/json".parse().unwrap());
    headers.insert("NS", params.ns.parse().unwrap());
    headers.insert("DB", params.db.parse().unwrap());

    client
        .post(params.url.to_owned())
        .basic_auth(params.username.to_owned(), Some(params.password.to_owned()))
        .headers(headers.to_owned())
        .body(query)
        .send()
        .await
        .unwrap()
}

async fn execute_transaction(params: &SurrealDbQueryParams, inner_query: String) -> Response {
    let query = format!(
        "BEGIN TRANSACTION;

{}

COMMIT TRANSACTION;",
        inner_query
    );

    execute_query(params, query).await
}

fn has_error(data: &EmptySurrealDbResponse) -> bool {
    data.iter().any(|r| r.status != "OK")
}

pub async fn main(
    up: Option<String>,
    url: Option<String>,
    ns: Option<String>,
    db: Option<String>,
    username: Option<String>,
    password: Option<String>,
) {
    let url = url.unwrap_or("http://127.0.0.1:8000/sql".to_owned());

    let username = username.unwrap_or("root".to_owned());
    let password = password.unwrap_or("root".to_owned());

    let ns = ns.unwrap_or("test".to_owned());
    let db = db.unwrap_or("test".to_owned());

    let query_params = SurrealDbQueryParams {
        url,
        ns,
        db,
        username,
        password,
    };

    let response = execute_query(&query_params, "SELECT * FROM script_migration;".to_owned()).await;

    if response.status() != 200 {
        panic!("RPC error");
    }

    let data = response
        .json::<GetScriptMigrationsResponse>()
        .await
        .unwrap();

    if data[0].status != "OK" {
        panic!("RPC error");
    }

    let migrations_applied = &data[0].result.as_deref().unwrap();
    let mut migrations_applied = migrations_applied.iter().collect::<Vec<_>>();
    migrations_applied.sort_by_key(|m| &m.executed_at);

    let mut config = HashSet::new();
    config.insert(DirEntryAttr::Name);
    config.insert(DirEntryAttr::Path);
    config.insert(DirEntryAttr::IsFile);

    let schemas_files = fs_extra::dir::ls("schemas", &config).unwrap();
    let events_files = fs_extra::dir::ls("events", &config).unwrap();
    let migrations_files = fs_extra::dir::ls("migrations", &config).unwrap();

    // apply schemas
    let schema_definitions = schemas_files
        .items
        .iter()
        .map(|file| {
            let path = file.get(&DirEntryAttr::Path).unwrap();

            let path = match path {
                DirEntryValue::String(path) => path,
                _ => panic!("Cannot get path to schema files"),
            };

            fs_extra::file::read_to_string(path).unwrap()
        })
        .collect::<Vec<_>>()
        .join("\n");

    let response = execute_transaction(&query_params, schema_definitions.to_owned()).await;

    // TODO : find & display error
    if response.status() != 200 {
        panic!("RPC error");
    }

    let data = response.json::<ExecuteSchemaResponse>().await.unwrap();

    // TODO : find & display error
    if has_error(&data) {
        panic!("RPC error");
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
                _ => panic!("Cannot get path to schema files"),
            };

            fs_extra::file::read_to_string(path).unwrap()
        })
        .collect::<Vec<_>>()
        .join("\n");

    let response = execute_transaction(&query_params, event_definitions.to_owned()).await;

    if response.status() != 200 {
        panic!("RPC error");
    }

    let data = response.json::<ExecuteEventResponse>().await.unwrap();

    if has_error(&data) {
        panic!("RPC error");
    }

    println!("Event files successfully executed!");

    // create definition files
    let last_migration_applied = migrations_applied.last();

    const INITIAL_DEFINITION_FILEPATH: &str = "migrations/definitions/_initial.json";

    match last_migration_applied {
        Some(last_migration_applied) => {
            // create folder "migrations/definitions" if not exists
            let definition_folder = Path::new("migrations/definitions");
            if !definition_folder.exists() {
                fs_extra::dir::create(definition_folder, false).unwrap();
            }

            let initial_definition =
                fs_extra::file::read_to_string(INITIAL_DEFINITION_FILEPATH).unwrap();
            let initial_definition =
                serde_json::from_str::<definitions::SchemaMigrationDefinition>(&initial_definition)
                    .unwrap();

            // calculate new definition based on all definitions files
            let diff_definition_files =
                fs_extra::dir::ls("migrations/definitions", &config).unwrap();

            let definition_diffs = diff_definition_files
                .items
                .iter()
                .filter(|file| {
                    let name = file.get(&DirEntryAttr::Name).unwrap();

                    let name = match name {
                        DirEntryValue::String(name) => name,
                        _ => panic!("Cannot get name to definition files"),
                    };

                    name != "_initial.json"
                })
                .take_while(|file| match file.get(&DirEntryAttr::Name).unwrap() {
                    DirEntryValue::String(name) => name != &last_migration_applied.script_name,
                    _ => panic!("Cannot get name to definition files"),
                })
                .map(|file| {
                    let path = file.get(&DirEntryAttr::Path).unwrap();

                    let path = match path {
                        DirEntryValue::String(path) => path,
                        _ => panic!("Cannot get path to definition files"),
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
            let definition_folder = Path::new("migrations/definitions");
            if !definition_folder.exists() {
                fs_extra::dir::create(definition_folder, false).unwrap();
            }

            let current_definition = definitions::SchemaMigrationDefinition {
                schemas: schema_definitions,
                events: event_definitions,
            };

            let serialized_definition = serde_json::to_string(&current_definition).unwrap();

            fs_extra::file::write_all(&INITIAL_DEFINITION_FILEPATH, &serialized_definition)
                .unwrap();
        }
    }

    // filter migrations not already applied & apply migrations
    for migration_file in migrations_files.items {
        let name = migration_file.get(&DirEntryAttr::Name).unwrap();
        let path = migration_file.get(&DirEntryAttr::Path).unwrap();
        let is_file = migration_file.get(&DirEntryAttr::IsFile).unwrap();

        let is_file = match is_file {
            DirEntryValue::Boolean(is_file) => is_file,
            _ => panic!("Cannot detect if the migration file is a file or a folder"),
        };

        if !is_file {
            continue;
        }

        let name = match name {
            DirEntryValue::String(name) => name,
            _ => panic!("Cannot get name of the migration file"),
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
            _ => panic!("Cannot get path to migration files"),
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

        let response = execute_transaction(&query_params, query).await;

        if response.status() != 200 {
            panic!("RPC error");
        }

        let data = response.json::<ExecuteMigrationResponse>().await.unwrap();

        if has_error(&data) {
            panic!("RPC error");
        }
    }

    println!("Migration files successfully executed!");
}
