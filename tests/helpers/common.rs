use anyhow::{anyhow, Context, Result};
use assert_cmd::Command;
use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
    process::{Child, Stdio},
};
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Surreal,
};
use surrealdb_migrations::SurrealdbConfiguration;

pub fn clear_files_dir() -> Result<()> {
    let files_dir = std::path::Path::new("tests-files");

    if files_dir.exists() {
        std::fs::remove_dir_all(files_dir)?;
    }

    Ok(())
}

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

pub async fn check_surrealdb_empty() -> Result<()> {
    let db_configuration = SurrealdbConfiguration::default();

    let client = create_surrealdb_client(&db_configuration).await?;

    let mut response = client.query("INFO FOR DB;").await?;

    let result: Option<SurrealdbTableDefinitions> = response.take("tb")?;
    let table_definitions = result.context("Failed to get table definitions")?;

    if table_definitions.len() > 0 {
        return Err(anyhow!("SurrealDB instance is not empty"));
    }

    Ok(())
}

type SurrealdbTableDefinitions = HashMap<String, String>;

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

    let client = create_surrealdb_connection(url.clone()).await?;
    sign_in(username.clone(), password.clone(), &client).await?;
    set_namespace_and_database(ns.clone(), db.clone(), &client).await?;

    Ok(client)
}

async fn create_surrealdb_connection(
    url: Option<String>,
) -> Result<Surreal<Client>, surrealdb::Error> {
    let url = url.unwrap_or("localhost:8000".to_owned());

    Surreal::new::<Ws>(url.to_owned()).await
}

async fn sign_in(
    username: Option<String>,
    password: Option<String>,
    client: &Surreal<Client>,
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
    client: &Surreal<Client>,
) -> Result<(), surrealdb::Error> {
    let ns = ns.unwrap_or("test".to_owned());
    let db = db.unwrap_or("test".to_owned());

    client.use_ns(ns.to_owned()).use_db(db.to_owned()).await
}

pub fn scaffold_empty_template() -> Result<()> {
    scaffold_template("empty")
}

pub fn scaffold_blog_template() -> Result<()> {
    scaffold_template("blog")
}

fn scaffold_template(template_name: &str) -> Result<()> {
    let mut cmd = create_cmd()?;
    cmd.arg("scaffold").arg("template").arg(template_name);
    cmd.assert().try_success()?;

    Ok(())
}

pub fn apply_migrations() -> Result<()> {
    let mut cmd = create_cmd()?;
    cmd.arg("apply");
    cmd.assert().try_success()?;

    Ok(())
}

pub fn apply_migrations_up_to(name: &str) -> Result<()> {
    let mut cmd = create_cmd()?;
    cmd.arg("apply").arg("--up").arg(name);
    cmd.assert().try_success()?;

    Ok(())
}

pub fn create_cmd() -> Result<Command> {
    let cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    Ok(cmd)
}

pub fn empty_folder(folder: &str) -> Result<()> {
    let migrations_files_dir = std::path::Path::new(folder);

    if migrations_files_dir.exists() {
        std::fs::remove_dir_all(migrations_files_dir)?;
        std::fs::create_dir(migrations_files_dir)?;
    }

    Ok(())
}

pub fn remove_folder(folder: &str) -> Result<()> {
    let migrations_files_dir = std::path::Path::new(folder);

    if migrations_files_dir.exists() {
        std::fs::remove_dir_all(migrations_files_dir)?;
    }

    Ok(())
}

pub fn add_new_schema_file() -> Result<()> {
    let schemas_files_dir = std::path::Path::new("tests-files/schemas");

    if schemas_files_dir.exists() {
        let category_schema_file = schemas_files_dir.join("category.surql");
        const CATEGORY_CONTENT: &str = "DEFINE TABLE category SCHEMALESS;

DEFINE FIELD name ON category TYPE string;
DEFINE FIELD created_at ON comment TYPE datetime VALUE $before OR time::now();";

        std::fs::write(category_schema_file, CATEGORY_CONTENT)?;
    }

    Ok(())
}

pub fn get_first_migration_name() -> Result<String> {
    let first_migration_file = get_first_migration_file()?;

    let first_migration_name = first_migration_file
        .file_stem()
        .ok_or_else(|| anyhow!("Could not get file stem"))?
        .to_str()
        .ok_or_else(|| anyhow!("Could not convert file stem to str"))?
        .to_owned();

    Ok(first_migration_name)
}

pub fn get_first_migration_file() -> Result<PathBuf> {
    let migrations_files_dir = std::path::Path::new("tests-files/migrations");

    let mut migration_files = fs::read_dir(migrations_files_dir)?
        .map(|entry| -> io::Result<PathBuf> { Ok(entry?.path()) })
        .collect::<Result<Vec<PathBuf>, io::Error>>()?;

    migration_files.sort_by(|a, b| {
        a.file_name()
            .unwrap_or_default()
            .cmp(&b.file_name().unwrap_or_default())
    });

    let first_migration_file = migration_files
        .first()
        .ok_or_else(|| anyhow!("No migration files found"))?;

    Ok(first_migration_file.to_path_buf())
}

pub fn are_folders_equivalent(folder_one: &str, folder_two: &str) -> Result<bool> {
    let is_different = dir_diff::is_different(folder_one, folder_two);

    match is_different {
        Ok(is_different) => {
            let are_equivalent = !is_different;
            Ok(are_equivalent)
        }
        Err(error) => Err(anyhow!("Cannot compare folders. {:?}", error)),
    }
}

pub fn is_empty_folder(folder: &str) -> Result<bool> {
    let dir = Path::new(folder).read_dir()?;
    let nubmer_of_files = dir.count();

    Ok(nubmer_of_files == 0)
}

pub fn is_file_exists(file_path: &str) -> Result<bool> {
    let file = Path::new(file_path);
    let is_file_exists = file.try_exists()?;

    Ok(is_file_exists)
}
