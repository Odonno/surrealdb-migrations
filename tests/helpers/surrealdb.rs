use anyhow::{anyhow, Context, Result};
use std::{
    collections::HashMap,
    process::{Child, Stdio},
    thread, time,
};
use surrealdb::{
    engine::any::{connect, Any},
    opt::auth::Root,
    Surreal,
};

use crate::helpers::SurrealdbConfiguration;

pub fn run_with_surreal_instance<F>(function: F) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    run_with_surreal_instance_with_params(function, "root", "root")
}

pub fn run_with_surreal_instance_with_admin_user<F>(function: F) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    run_with_surreal_instance_with_params(function, "admin", "admin")
}

fn run_with_surreal_instance_with_params<F>(
    function: F,
    username: &str,
    password: &str,
) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    let mut child_process = start_surreal_process(username, password)?;

    while !is_surreal_ready()? {
        thread::sleep(time::Duration::from_millis(100));
    }

    let result = function();

    match child_process.kill() {
        Ok(_) => result,
        Err(error) => Err(anyhow!("Failed to kill child process: {}", error)),
    }
}

pub async fn run_with_surreal_instance_async<F>(function: F) -> Result<()>
where
    F: FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>>,
{
    run_with_surreal_instance_with_params_async(function, "root", "root").await
}

pub async fn run_with_surreal_instance_with_admin_user_async<F>(function: F) -> Result<()>
where
    F: FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>>,
{
    run_with_surreal_instance_with_params_async(function, "admin", "admin").await
}

async fn run_with_surreal_instance_with_params_async<F>(
    function: F,
    username: &str,
    password: &str,
) -> Result<()>
where
    F: FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>>,
{
    let mut child_process = start_surreal_process(username, password)?;

    while !is_surreal_ready()? {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    let result = function().await;

    match child_process.kill() {
        Ok(_) => result,
        Err(error) => Err(anyhow!("Failed to kill child process: {}", error)),
    }
}

fn start_surreal_process(username: &str, password: &str) -> Result<Child> {
    let child_process = std::process::Command::new("surreal")
        .arg("start")
        .arg("--user")
        .arg(username)
        .arg("--pass")
        .arg(password)
        .arg("memory")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    Ok(child_process)
}

fn is_surreal_ready() -> Result<bool> {
    let child_process = std::process::Command::new("surreal")
        .arg("isready")
        .arg("--conn")
        .arg("http://localhost:8000")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    let output = child_process.wait_with_output()?;

    Ok(output.status.success())
}

pub async fn check_surrealdb_empty() -> Result<()> {
    let db_configuration = SurrealdbConfiguration::default();

    let client = create_surrealdb_client(&db_configuration).await?;

    let mut response = client.query("INFO FOR DB;").await?;

    type SurrealdbTableDefinitions = HashMap<String, String>;

    let result: Option<SurrealdbTableDefinitions> = response.take("tb")?;
    let table_definitions = result.context("Failed to get table definitions")?;

    if table_definitions.len() > 0 {
        return Err(anyhow!("SurrealDB instance is not empty"));
    }

    Ok(())
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
) -> Result<(), surrealdb::Error> {
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
