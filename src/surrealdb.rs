use color_eyre::eyre::{ContextCompat, Result, eyre};
use itertools::Itertools;
use serde::Deserialize;
use std::collections::HashMap;
use surrealdb::{Connection, Surreal};

use crate::{constants::SCRIPT_MIGRATION_TABLE_NAME, models::ScriptMigration};

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
    let response = client
        .query(surrealdb::sql::statements::InfoStatement::Db(false, None))
        .await?;

    let mut response = response.check()?;

    let result: Option<SurrealdbTableDefinitions> = response.take("tables")?;
    let table_definitions = result.context("Failed to get table definitions")?;

    Ok(table_definitions)
}

#[derive(Debug, Deserialize)]
pub struct SurrealdbTableDefinition {
    pub fields: HashMap<String, String>,
}

pub async fn get_surrealdb_table_definition<C: Connection>(
    client: &Surreal<C>,
    table: &str,
) -> Result<SurrealdbTableDefinition> {
    let response = client
        .query(surrealdb::sql::statements::InfoStatement::Tb(
            table.into(),
            false,
            None,
        ))
        .await?;

    let mut response = response.check()?;

    let result: Option<SurrealdbTableDefinition> = response.take(0)?;
    let table_definition = result.context(format!(
        "Failed to get table definition for table '{table}'"
    ))?;

    Ok(table_definition)
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

pub fn is_define_checksum_statement(statement: &surrealdb::sql::Statement) -> bool {
    match statement {
        surrealdb::sql::Statement::Define(surrealdb::sql::statements::DefineStatement::Field(
            define_field_statement,
        )) => {
            define_field_statement.name.to_string() == "checksum"
                && define_field_statement.what.0 == SCRIPT_MIGRATION_TABLE_NAME
        }
        _ => false,
    }
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
