use anyhow::Result;
use surrealdb::{
    engine::remote::ws::{Client, Ws, Wss},
    opt::auth::Root,
    Surreal,
};

use crate::{config, models::ScriptMigration};

pub async fn create_surrealdb_client(
    url: Option<String>,
    ns: Option<String>,
    db: Option<String>,
    username: Option<String>,
    password: Option<String>,
) -> Result<Surreal<Client>> {
    let db_config = config::retrieve_db_config();

    let client = create_surrealdb_connection(url, &db_config).await?;
    sign_in(username, password, &db_config, &client).await?;
    set_namespace_and_database(ns, db, &db_config, &client).await?;

    Ok(client)
}

async fn create_surrealdb_connection(
    url: Option<String>,
    db_config: &config::DbConfig,
) -> Result<Surreal<Client>, surrealdb::Error> {
    let url = url
        .or(db_config.url.to_owned())
        .unwrap_or("localhost:8000".to_owned());

    if is_local_instance(url.to_owned()) {
        Surreal::new::<Ws>(url.to_owned()).await
    } else {
        Surreal::new::<Wss>(url.to_owned()).await
    }
}

fn is_local_instance(url: String) -> bool {
    url == "localhost" || url.starts_with("localhost:")
}

async fn sign_in(
    username: Option<String>,
    password: Option<String>,
    db_config: &config::DbConfig,
    client: &Surreal<Client>,
) -> Result<(), surrealdb::Error> {
    let username = username
        .or(db_config.username.to_owned())
        .unwrap_or("root".to_owned());
    let password = password
        .or(db_config.password.to_owned())
        .unwrap_or("root".to_owned());

    client
        .signin(Root {
            username: &username,
            password: &password,
        })
        .await
}

async fn set_namespace_and_database(
    ns: Option<String>,
    db: Option<String>,
    db_config: &config::DbConfig,
    client: &Surreal<Client>,
) -> Result<(), surrealdb::Error> {
    let ns = ns.or(db_config.ns.to_owned()).unwrap_or("test".to_owned());
    let db = db.or(db_config.db.to_owned()).unwrap_or("test".to_owned());

    client.use_ns(ns.to_owned()).use_db(db.to_owned()).await
}

pub async fn list_script_migration_ordered_by_execution_date(
    client: &Surreal<Client>,
) -> Result<Vec<ScriptMigration>> {
    let mut result = list_script_migration(client).await?;
    result.sort_by_key(|m| m.executed_at.clone());

    Ok(result)
}

async fn list_script_migration(client: &Surreal<Client>) -> Result<Vec<ScriptMigration>> {
    let result = client.select("script_migration").await?;
    Ok(result)
}

pub fn within_transaction(inner_query: String) -> String {
    format!(
        "BEGIN TRANSACTION;

{}

COMMIT TRANSACTION;",
        inner_query
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn localhost_should_be_local_instance() {
        let url = "localhost";
        assert!(is_local_instance(url.to_owned()));
    }

    #[test]
    fn localhost_on_port_8000_should_be_local_instance() {
        let url = "localhost:8000";
        assert!(is_local_instance(url.to_owned()));
    }

    #[test]
    fn localhost_without_port_value_should_be_local_instance() {
        let url = "localhost:";
        assert!(is_local_instance(url.to_owned()));
    }

    #[test]
    fn remote_should_not_be_local_instance() {
        let url = "cloud.surrealdb.com";
        assert!(!is_local_instance(url.to_owned()));
    }
}
