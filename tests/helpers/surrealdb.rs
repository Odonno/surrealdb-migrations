use color_eyre::eyre::{ContextCompat, Result};
use std::collections::HashMap;
use surrealdb::{
    engine::any::{connect, Any},
    opt::{
        auth::{Jwt, Root},
        capabilities::Capabilities,
        Config,
    },
    sql::Thing,
    Surreal,
};

use crate::helpers::SurrealdbConfiguration;

use super::DbInstance;

pub async fn is_surreal_db_empty(ns: Option<String>, db: Option<String>) -> Result<bool> {
    let db_configuration = SurrealdbConfiguration {
        ns,
        db,
        ..Default::default()
    };

    let table_definitions = get_surrealdb_table_definitions(Some(db_configuration)).await?;

    Ok(table_definitions.is_empty())
}

type SurrealdbTableDefinitions = HashMap<String, String>;

pub async fn get_surrealdb_table_definitions(
    db_configuration: Option<SurrealdbConfiguration>,
) -> Result<SurrealdbTableDefinitions> {
    let db_configuration = db_configuration.unwrap_or_default();
    let client = create_surrealdb_client(&db_configuration).await?;

    let mut response = client
        .query(surrealdb::sql::statements::InfoStatement::Db(false, None))
        .await?;

    let result: Option<SurrealdbTableDefinitions> = response.take("tables")?;
    let table_definitions = result.context("Failed to get table definitions")?;

    Ok(table_definitions)
}

pub async fn get_surrealdb_table_exists(
    db_configuration: Option<SurrealdbConfiguration>,
    table: &str,
) -> Result<bool> {
    let tables = get_surrealdb_table_definitions(db_configuration).await?;
    Ok(tables.contains_key(table))
}

pub async fn is_surreal_table_empty(
    ns_db: Option<(&str, &str)>,
    table: &'static str,
) -> Result<bool> {
    let mut db_configuration = SurrealdbConfiguration::default();
    if let Some((ns, db)) = ns_db {
        db_configuration.ns = Some(ns.to_string());
        db_configuration.db = Some(db.to_string());
    }

    if !get_surrealdb_table_exists(Some(db_configuration.clone()), table).await? {
        return Ok(true);
    }

    let client = create_surrealdb_client(&db_configuration).await?;

    let mut response = client
        .query("SELECT VALUE id FROM type::table($table);")
        .bind(("table", table))
        .await?;

    let records: Vec<Thing> = response.take(0)?;

    Ok(records.is_empty())
}

pub async fn get_surrealdb_records<T: for<'de> serde::de::Deserialize<'de>>(
    ns: String,
    db: String,
    table: String,
) -> Result<Vec<T>> {
    let db_configuration = SurrealdbConfiguration {
        ns: Some(ns),
        db: Some(db),
        ..Default::default()
    };

    let client = create_surrealdb_client(&db_configuration).await?;
    let records: Vec<T> = client.select(table).await?;

    Ok(records)
}

pub async fn get_surrealdb_record<T: for<'de> serde::de::Deserialize<'de>>(
    ns: String,
    db: String,
    table: String,
    id: String,
) -> Result<Option<T>> {
    let db_configuration = SurrealdbConfiguration {
        ns: Some(ns),
        db: Some(db),
        ..Default::default()
    };

    let client = create_surrealdb_client(&db_configuration).await?;
    let record: Option<T> = client.select((table, id)).await?;

    Ok(record)
}

pub async fn create_surrealdb_client(
    db_configuration: &SurrealdbConfiguration,
) -> Result<Surreal<Any>> {
    let SurrealdbConfiguration {
        address,
        username,
        password,
        ns,
        db,
    } = db_configuration;

    let client = create_surrealdb_connection(address.clone()).await?;
    sign_in(username.clone(), password.clone(), &client).await?;
    set_namespace_and_database(ns.clone(), db.clone(), &client).await?;

    Ok(client)
}

async fn create_surrealdb_connection(
    address: Option<String>,
) -> Result<Surreal<Any>, surrealdb::Error> {
    let address = address.unwrap_or(String::from("ws://localhost:8000"));

    let config =
        Config::new().capabilities(Capabilities::all().with_all_experimental_features_allowed());

    connect((address, config)).await
}

async fn sign_in(
    username: Option<String>,
    password: Option<String>,
    client: &Surreal<Any>,
) -> Result<Jwt, surrealdb::Error> {
    let username = username.unwrap_or("root".to_owned());
    let password = password.unwrap_or("root".to_owned());

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
    client: &Surreal<Any>,
) -> Result<(), surrealdb::Error> {
    let ns = ns.unwrap_or("test".to_owned());
    let db = db.unwrap_or("test".to_owned());

    let response = client
        .query(format!("DEFINE NAMESPACE IF NOT EXISTS `{ns}`;"))
        .await?;
    response.check()?;
    client.use_ns(ns.to_owned()).await?;

    let response = client
        .query(format!("DEFINE DATABASE IF NOT EXISTS `{db}`;"))
        .await?;
    response.check()?;
    client.use_db(db.to_owned()).await?;

    Ok(())
}

pub async fn remove_features_ns() -> Result<()> {
    let db_configuration = SurrealdbConfiguration::default();

    let client = create_surrealdb_client(&db_configuration).await?;
    client.query("REMOVE NAMESPACE features;").await?;

    Ok(())
}

pub async fn execute_sql_statements(query: &str, db_instance: DbInstance, db: &str) -> Result<()> {
    let client = create_surrealdb_client(&SurrealdbConfiguration::from(db_instance, db)).await?;
    client.query(query).await?;

    Ok(())
}
