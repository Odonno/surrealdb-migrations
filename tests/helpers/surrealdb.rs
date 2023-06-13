use anyhow::{Context, Result};
use std::collections::HashMap;
use surrealdb::{
    engine::any::{connect, Any},
    opt::auth::{Jwt, Root},
    sql::Thing,
    Surreal,
};

use crate::helpers::SurrealdbConfiguration;

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
    let db_configuration = db_configuration.unwrap_or(SurrealdbConfiguration::default());
    let client = create_surrealdb_client(&db_configuration).await?;

    let mut response = client.query("INFO FOR DB;").await?;

    let result: Option<SurrealdbTableDefinitions> = response.take("tables")?;
    let table_definitions = result.context("Failed to get table definitions")?;

    Ok(table_definitions)
}

pub async fn is_surreal_table_empty(ns_db: Option<(&str, &str)>, table: &str) -> Result<bool> {
    let mut db_configuration = SurrealdbConfiguration::default();
    if let Some((ns, db)) = ns_db {
        db_configuration.ns = Some(ns.to_string());
        db_configuration.db = Some(db.to_string());
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
        url,
        username,
        password,
        ns,
        db,
    } = db_configuration;

    let client = create_surrealdb_connection(url.clone(), address.clone()).await?;
    sign_in(username.clone(), password.clone(), &client).await?;
    set_namespace_and_database(ns.clone(), db.clone(), &client).await?;

    Ok(client)
}

async fn create_surrealdb_connection(
    url: Option<String>,
    address: Option<String>,
) -> Result<Surreal<Any>, surrealdb::Error> {
    let url = url.unwrap_or("localhost:8000".to_owned());
    let address = address.unwrap_or(format!("ws://{}", url));

    connect(address).await
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

    client.use_ns(ns.to_owned()).use_db(db.to_owned()).await
}

pub async fn remove_features_ns() -> Result<()> {
    let db_configuration = SurrealdbConfiguration::default();

    let client = create_surrealdb_client(&db_configuration).await?;
    client.query("REMOVE NAMESPACE features;").await?;

    Ok(())
}
