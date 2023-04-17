use anyhow::Result;
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Surreal,
};

use crate::config;

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
