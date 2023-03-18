use std::collections::HashSet;

use fs_extra::dir::{DirEntryAttr, DirEntryValue};
use reqwest::{
    header::{HeaderMap, ACCEPT},
    Response,
};
use serde::{Deserialize, Serialize};

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

    let response = execute_query(&query_params, "SELECT * FROM script_migration".to_owned()).await;

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

    let migrations_applied = &data[0].result.as_ref().unwrap();

    let mut config = HashSet::new();
    config.insert(DirEntryAttr::Name);
    config.insert(DirEntryAttr::Path);

    let schemas_files = fs_extra::dir::ls("schemas", &config).unwrap();
    let events_files = fs_extra::dir::ls("events", &config).unwrap();
    let migrations_files = fs_extra::dir::ls("migrations", &config).unwrap();

    // apply schemas
    for schema_file in schemas_files.items {
        let path = schema_file.get(&DirEntryAttr::Path).unwrap();

        let path = match path {
            DirEntryValue::String(path) => path,
            _ => panic!("Cannot get path to schema files"),
        };

        let query = fs_extra::file::read_to_string(path).unwrap();
        let response = execute_query(&query_params, query).await;

        if response.status() != 200 {
            panic!("RPC error");
        }

        let data = response.json::<ExecuteSchemaResponse>().await.unwrap();

        if has_error(&data) {
            panic!("RPC error");
        }
    }

    println!("Schema files successfully executed!");

    // apply events
    for event_file in events_files.items {
        let path = event_file.get(&DirEntryAttr::Path).unwrap();

        let path = match path {
            DirEntryValue::String(path) => path,
            _ => panic!("Cannot get path to event files"),
        };

        let query = fs_extra::file::read_to_string(path).unwrap();
        let response = execute_query(&query_params, query).await;

        if response.status() != 200 {
            panic!("RPC error");
        }

        let data = response.json::<ExecuteEventResponse>().await.unwrap();

        if has_error(&data) {
            panic!("RPC error");
        }
    }

    println!("Event files successfully executed!");

    // filter migrations not already applied & apply migrations
    for migration_file in migrations_files.items {
        let name = migration_file.get(&DirEntryAttr::Name).unwrap();
        let path = migration_file.get(&DirEntryAttr::Path).unwrap();

        let name = match name {
            DirEntryValue::String(name) => name,
            _ => panic!("Cannot get path to migration files"),
        };

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
CREATE script_migration SET script_name = '{}'",
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

        println!("Migration {} applied", script_display_name);
    }

    println!("Migration files successfully executed!");
}
