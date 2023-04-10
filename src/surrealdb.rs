use std::process;
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
) -> Surreal<Client> {
    let db_config = config::retrieve_db_config();

    let url = url.or(db_config.url).unwrap_or("localhost:8000".to_owned());

    let connection = Surreal::new::<Ws>(url.to_owned()).await;

    if let Err(error) = connection {
        eprintln!("{}", error);
        process::exit(1);
    }

    let client = connection.unwrap();

    let username = username.or(db_config.username).unwrap_or("root".to_owned());
    let password = password.or(db_config.password).unwrap_or("root".to_owned());

    client
        .signin(Root {
            username: &username,
            password: &password,
        })
        .await
        .unwrap();

    let ns = ns.or(db_config.ns).unwrap_or("test".to_owned());
    let db = db.or(db_config.db).unwrap_or("test".to_owned());

    client
        .use_ns(ns.to_owned())
        .use_db(db.to_owned())
        .await
        .unwrap();

    client
}
