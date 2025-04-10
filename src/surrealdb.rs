use color_eyre::eyre::{eyre, ContextCompat, Result};
use itertools::Itertools;
use std::collections::HashMap;
use std::path::Path;
use surrealdb::{
    engine::any::{connect, Any},
    opt::{
        auth::{Jwt, Root},
        capabilities::Capabilities,
        Config,
    },
    Connection, Surreal,
};

use crate::{
    config, constants::SCRIPT_MIGRATION_TABLE_NAME, input::SurrealdbConfiguration,
    models::ScriptMigration,
};

#[allow(dead_code)]
pub async fn create_surrealdb_client(
    config_file: Option<&Path>,
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

    let config =
        Config::new().capabilities(Capabilities::all().with_all_experimental_features_allowed());

    connect((address, config)).await
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

    client.query(format!("DEFINE NAMESPACE `{ns}`")).await?;
    client.use_ns(ns.to_owned()).await?;

    client.query(format!("DEFINE DATABASE `{db}`")).await?;
    client.use_db(db.to_owned()).await?;

    Ok(())
}

pub async fn get_surrealdb_table_exists<C: Connection>(
    client: &Surreal<C>,
    table: &str,
) -> Result<bool> {
    let tables = get_surrealdb_table_definitions(client).await?;
    Ok(tables.contains_key(table))
}

type SurrealdbTableDefinitions = HashMap<String, String>;

pub async fn get_surrealdb_table_definitions<C: Connection>(
    client: &Surreal<C>,
) -> Result<SurrealdbTableDefinitions> {
    let mut response = client.query("INFO FOR DB;").await?;

    let result: Option<SurrealdbTableDefinitions> = response.take("tables")?;
    let table_definitions = result.context("Failed to get table definitions")?;

    Ok(table_definitions)
}

pub async fn list_script_migration_ordered_by_execution_date<C: Connection>(
    client: &Surreal<C>,
) -> Result<Vec<ScriptMigration>> {
    if get_surrealdb_table_exists(client, SCRIPT_MIGRATION_TABLE_NAME).await? {
        let mut result = list_script_migration(client).await?;
        result.sort_by_key(|m| m.executed_at.clone());
        Ok(result)
    } else {
        Ok(vec![])
    }
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
            let end_result = response_result.and_then(|response| response.check());

            let first_error = end_result.err().context("Error on rollback")?;
            let is_rollback_success = first_error
                .to_string()
                .contains("The query was not executed due to a cancelled transaction");

            if is_rollback_success {
                Ok(())
            } else {
                Err(eyre!(first_error))
            }
        }
        TransactionAction::Commit => {
            let mut response = response_result?;

            let errors = response.take_errors();
            if !errors.is_empty() {
                const FAILED_TRANSACTION_ERROR: &str =
                    "The query was not executed due to a failed transaction";

                let is_failed_transaction = errors
                    .values()
                    .any(|e| e.to_string() == FAILED_TRANSACTION_ERROR);

                let initial_error_messages = match is_failed_transaction {
                    true => {
                        vec![FAILED_TRANSACTION_ERROR.to_string()]
                    }
                    false => vec![],
                };

                let error_messages = errors
                    .values()
                    .map(|e| e.to_string())
                    .filter(|e| e != FAILED_TRANSACTION_ERROR)
                    .collect_vec();
                let error_messages = initial_error_messages
                    .into_iter()
                    .chain(error_messages.into_iter())
                    .collect_vec();

                return Err(eyre!(error_messages.join("\n")));
            }

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
