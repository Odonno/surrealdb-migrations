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
    let SurrealdbConfiguration {
        address,
        username,
        password,
        ns,
        db,
    } = db_configuration;

    let db_config = config::retrieve_db_config(config_file);

    let client = create_surrealdb_connection(address.clone(), &db_config).await?;
    sign_in(username.clone(), password.clone(), &db_config, &client).await?;
    set_namespace_and_database(ns.clone(), db.clone(), &db_config, &client).await?;

    Ok(client)
}

async fn create_surrealdb_connection(
    address: Option<String>,
    db_config: &config::DbConfig,
) -> Result<Surreal<Any>, surrealdb::Error> {
    let address = address
        .or(db_config.address.to_owned())
        .unwrap_or(String::from("ws://localhost:8000"));

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

    let mut ns_statement = surrealdb::sql::statements::DefineNamespaceStatement::default();
    ns_statement.name = ns.to_string().into();
    ns_statement.if_not_exists = true;
    let ns_statement = surrealdb::sql::statements::DefineStatement::Namespace(ns_statement);

    let response = client.query(ns_statement).await?;
    response.check()?;
    client.use_ns(ns.to_string()).await?;

    let mut db_statement = surrealdb::sql::statements::DefineDatabaseStatement::default();
    db_statement.name = db.to_string().into();
    db_statement.if_not_exists = true;
    let db_statement = surrealdb::sql::statements::DefineStatement::Database(db_statement);

    let response = client.query(db_statement).await?;
    response.check()?;
    client.use_db(db.to_string()).await?;

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
    let mut response = client
        .query(surrealdb::sql::statements::InfoStatement::Db(false, None))
        .await?;

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

pub fn parse_statements(query_str: &str) -> Result<surrealdb::sql::Query> {
    let query = ::surrealdb::syn::parse_with_capabilities(
        query_str,
        &::surrealdb::dbs::Capabilities::all()
            .with_experimental(::surrealdb::dbs::capabilities::Targets::All),
    )?;

    Ok(query)
}

pub async fn apply_in_transaction<C: Connection>(
    client: &Surreal<C>,
    statements: Vec<surrealdb::sql::Statement>,
    action: TransactionAction,
) -> Result<()> {
    let mut statements = statements.clone();

    statements.insert(
        0,
        surrealdb::sql::Statement::Begin(surrealdb::sql::statements::BeginStatement::default()),
    );

    let end_statement = match action {
        TransactionAction::Commit => {
            surrealdb::sql::Statement::Commit(surrealdb::sql::statements::CommitStatement::default())
        }
        TransactionAction::Rollback => {
            surrealdb::sql::Statement::Cancel(surrealdb::sql::statements::CancelStatement::default())
        }
    };
    statements.push(end_statement);

    let response_result = client.query(statements).await;

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

#[derive(Debug, PartialEq)]
pub enum TransactionAction {
    Commit,
    Rollback,
}
