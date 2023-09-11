use anyhow::{anyhow, Context, Result};
use surrealdb::{
    engine::any::{connect, Any},
    opt::auth::{Jwt, Root},
    Connection, Response, Surreal,
};

use crate::{
    config, constants::SCRIPT_MIGRATION_TABLE_NAME, input::SurrealdbConfiguration,
    models::ScriptMigration,
};

#[allow(dead_code)]
pub async fn create_surrealdb_client(
    config_file: Option<&str>,
    db_configuration: &SurrealdbConfiguration,
) -> Result<Surreal<Any>> {
    #[allow(deprecated)]
    let SurrealdbConfiguration {
        address,
        url,
        username,
        password,
        ns,
        db,
    } = db_configuration;

    let db_config = config::retrieve_db_config(config_file);

    let client = create_surrealdb_connection(url.clone(), address.clone(), &db_config).await?;
    sign_in(username.clone(), password.clone(), &db_config, &client).await?;
    set_namespace_and_database(ns.clone(), db.clone(), &db_config, &client).await?;

    Ok(client)
}

async fn create_surrealdb_connection(
    url: Option<String>,
    address: Option<String>,
    db_config: &config::DbConfig,
) -> Result<Surreal<Any>, surrealdb::Error> {
    let url = url
        .or(db_config.url.to_owned())
        .unwrap_or("localhost:8000".to_owned());

    let address = address
        .or(db_config.address.to_owned())
        .unwrap_or(format!("ws://{}", url));

    connect(address).await
}

async fn sign_in(
    username: Option<String>,
    password: Option<String>,
    db_config: &config::DbConfig,
    client: &Surreal<Any>,
) -> Result<Jwt, surrealdb::Error> {
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
    client: &Surreal<Any>,
) -> Result<(), surrealdb::Error> {
    let ns = ns.or(db_config.ns.to_owned()).unwrap_or("test".to_owned());
    let db = db.or(db_config.db.to_owned()).unwrap_or("test".to_owned());

    client.use_ns(ns.to_owned()).use_db(db.to_owned()).await
}

pub async fn list_script_migration_ordered_by_execution_date<C: Connection>(
    client: &Surreal<C>,
) -> Result<Vec<ScriptMigration>> {
    let mut result = list_script_migration(client).await?;
    result.sort_by_key(|m| m.executed_at.clone());

    Ok(result)
}

async fn list_script_migration<C: Connection>(client: &Surreal<C>) -> Result<Vec<ScriptMigration>> {
    let result = client.select(SCRIPT_MIGRATION_TABLE_NAME).await?;
    Ok(result)
}

pub async fn apply_in_transaction<C: Connection>(
    client: &Surreal<C>,
    inner_query: &String,
    action: TransactionAction,
) -> Result<()> {
    let query = format_transaction(inner_query.to_owned(), &action);
    let response_result = client.query(query).await;

    match action {
        TransactionAction::Rollback => {
            let end_result = response_result.and_then(|response: Response| response.check());

            let first_error = end_result.err().context("Error on rollback")?;
            let is_rollback_success = first_error
                .to_string()
                .contains("The query was not executed due to a cancelled transaction");

            if is_rollback_success {
                Ok(())
            } else {
                Err(anyhow!(first_error))
            }
        }
        TransactionAction::Commit => {
            let response = response_result?;
            response.check()?;
            Ok(())
        }
    }
}

fn format_transaction(inner_query: String, action: &TransactionAction) -> String {
    match action {
        TransactionAction::Commit => format_transaction_with_commit(inner_query),
        TransactionAction::Rollback => format_transaction_with_rollback(inner_query),
    }
}

#[derive(Debug, PartialEq)]
pub enum TransactionAction {
    Commit,
    Rollback,
}

fn format_transaction_with_commit(inner_query: String) -> String {
    format!(
        "BEGIN TRANSACTION;

{}

COMMIT TRANSACTION;",
        inner_query
    )
}

fn format_transaction_with_rollback(inner_query: String) -> String {
    format!(
        "BEGIN TRANSACTION;

{}

CANCEL TRANSACTION;",
        inner_query
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn within_transaction_should_return_string() {
        let inner_query = "DEFINE TABLE post SCHEMALESS;";
        let result = format_transaction(inner_query.to_owned(), &TransactionAction::Commit);

        assert_eq!(
            result,
            "BEGIN TRANSACTION;

DEFINE TABLE post SCHEMALESS;

COMMIT TRANSACTION;"
        );
    }

    #[test]
    fn within_rollback_should_return_string() {
        let inner_query = "DEFINE TABLE post SCHEMALESS;";
        let result = format_transaction(inner_query.to_owned(), &TransactionAction::Rollback);

        assert_eq!(
            result,
            "BEGIN TRANSACTION;

DEFINE TABLE post SCHEMALESS;

CANCEL TRANSACTION;"
        );
    }
}
