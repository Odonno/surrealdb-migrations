use color_eyre::eyre::Result;
use std::path::Path;
use surrealdb::{
    engine::any::{connect, Any},
    opt::{
        auth::{Jwt, Root},
        capabilities::Capabilities,
        Config,
    },
    Surreal,
};

use crate::input::SurrealdbConfiguration;

use super::db_config::{retrieve_db_config, DbConfig};

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

    let db_config = retrieve_db_config(config_file);

    let client = create_surrealdb_connection(address.clone(), &db_config).await?;
    sign_in(username.clone(), password.clone(), &db_config, &client).await?;
    set_namespace_and_database(ns.clone(), db.clone(), &db_config, &client).await?;

    Ok(client)
}

async fn create_surrealdb_connection(
    address: Option<String>,
    db_config: &DbConfig,
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
    db_config: &DbConfig,
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
    db_config: &DbConfig,
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
