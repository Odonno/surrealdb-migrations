use anyhow::{anyhow, Context, Result};
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Surreal,
};

use crate::{config, input::SurrealdbConfiguration, models::ScriptMigration};

pub async fn create_surrealdb_client(
    db_configuration: &SurrealdbConfiguration,
) -> Result<Surreal<Client>> {
    let SurrealdbConfiguration {
        url,
        username,
        password,
        ns,
        db,
    } = db_configuration;

    let db_config = config::retrieve_db_config();

    let client = create_surrealdb_connection(url.clone(), &db_config).await?;
    sign_in(username.clone(), password.clone(), &db_config, &client).await?;
    set_namespace_and_database(ns.clone(), db.clone(), &db_config, &client).await?;

    Ok(client)
}

async fn create_surrealdb_connection(
    url: Option<String>,
    db_config: &config::DbConfig,
) -> Result<Surreal<Client>, surrealdb::Error> {
    let url = url
        .or(db_config.url.to_owned())
        .unwrap_or("localhost:8000".to_owned());

    Surreal::new::<Ws>(url.to_owned()).await
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

pub async fn apply_in_transaction(
    client: &Surreal<Client>,
    inner_query: &String,
    action: TransactionAction,
) -> Result<()> {
    let query = format_transaction(inner_query.to_owned(), &action);
    let response = client.query(query).await?;

    match action {
        TransactionAction::Rollback => {
            let first_error = response.check().err().context("Error on rollback")?;
            let is_rollback_success = first_error.to_string()
                == "The query was not executed due to a cancelled transaction";

            if is_rollback_success {
                Ok(())
            } else {
                Err(anyhow!(first_error))
            }
        }
        TransactionAction::Commit => {
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
