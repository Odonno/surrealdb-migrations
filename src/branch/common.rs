use std::{fs, path::PathBuf};

use color_eyre::eyre::{Result};
use surrealdb::{engine::any::Any, Surreal};

use crate::{input::SurrealdbConfiguration, models::Branch, surrealdb::create_surrealdb_client};

use super::constants::{BRANCH_NS, BRANCH_TABLE, ORIGIN_BRANCH_NS};

#[allow(deprecated)]
pub async fn create_branching_feature_client(
    config_file: Option<&str>,
    db_configuration: &SurrealdbConfiguration,
) -> Result<Surreal<Any>> {
    const BRANCH_DATA_NS: &str = "features";
    const BRANCH_DATA_DB: &str = "branching";

    let branch_data_db_configuration = SurrealdbConfiguration {
        address: db_configuration.address.clone(),
        url: db_configuration.url.clone(),
        username: db_configuration.username.clone(),
        password: db_configuration.password.clone(),
        ns: Some(BRANCH_DATA_NS.to_owned()),
        db: Some(BRANCH_DATA_DB.to_owned()),
    };

    let client = create_surrealdb_client(config_file, &branch_data_db_configuration).await?;
    Ok(client)
}

#[allow(deprecated)]
pub async fn create_branch_client(
    config_file: Option<&str>,
    branch_name: &String,
    db_configuration: &SurrealdbConfiguration,
) -> Result<Surreal<Any>> {
    let branch_db_configuration = SurrealdbConfiguration {
        address: db_configuration.address.clone(),
        url: db_configuration.url.clone(),
        username: db_configuration.username.clone(),
        password: db_configuration.password.clone(),
        ns: Some(BRANCH_NS.to_owned()),
        db: Some(branch_name.to_owned()),
    };

    let client = create_surrealdb_client(config_file, &branch_db_configuration).await?;
    Ok(client)
}

#[allow(deprecated)]
pub async fn create_origin_branch_client(
    config_file: Option<&str>,
    branch_name: &String,
    db_configuration: &SurrealdbConfiguration,
) -> Result<Surreal<Any>> {
    let branch_db_configuration = SurrealdbConfiguration {
        address: db_configuration.address.clone(),
        url: db_configuration.url.clone(),
        username: db_configuration.username.clone(),
        password: db_configuration.password.clone(),
        ns: Some(ORIGIN_BRANCH_NS.to_owned()),
        db: Some(branch_name.to_owned()),
    };

    let client = create_surrealdb_client(config_file, &branch_db_configuration).await?;
    Ok(client)
}

#[allow(deprecated)]
pub async fn create_main_branch_client(
    config_file: Option<&str>,
    db_configuration: &SurrealdbConfiguration,
    branch: &Branch,
) -> Result<Surreal<Any>> {
    let main_branch_db_configuration = SurrealdbConfiguration {
        address: db_configuration.address.clone(),
        url: db_configuration.url.clone(),
        username: db_configuration.username.clone(),
        password: db_configuration.password.clone(),
        ns: Some(branch.from_ns.to_string()),
        db: Some(branch.from_db.to_string()),
    };

    let client = create_surrealdb_client(config_file, &main_branch_db_configuration).await?;
    Ok(client)
}

pub async fn retrieve_existing_branch_names(
    branching_feature_client: &Surreal<Any>,
) -> Result<Vec<String>> {
    let existing_branch_names: Vec<String> = branching_feature_client
        .query(format!("SELECT VALUE name FROM {}", BRANCH_TABLE))
        .await?
        .take(0)?;

    Ok(existing_branch_names)
}

pub fn remove_dump_file(dump_file_path: &PathBuf) -> Result<()> {
    fs::remove_file(dump_file_path)?;
    Ok(())
}
