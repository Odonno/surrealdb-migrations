use std::{
    fs,
    path::{Path, PathBuf},
};

use color_eyre::eyre::Result;
use surrealdb::{engine::any::Any, Surreal};

use crate::{
    input::SurrealdbConfiguration, models::Branch, runbin::surrealdb::create_surrealdb_client,
    surrealdb::get_surrealdb_table_exists,
};

use super::constants::{BRANCH_NS, BRANCH_TABLE, ORIGIN_BRANCH_NS};

pub async fn create_branching_feature_client(
    config_file: Option<&Path>,
    db_configuration: &SurrealdbConfiguration,
) -> Result<Surreal<Any>> {
    const BRANCH_DATA_NS: &str = "features";
    const BRANCH_DATA_DB: &str = "branching";

    let branch_data_db_configuration = SurrealdbConfiguration {
        address: db_configuration.address.clone(),
        username: db_configuration.username.clone(),
        password: db_configuration.password.clone(),
        ns: Some(BRANCH_DATA_NS.to_owned()),
        db: Some(BRANCH_DATA_DB.to_owned()),
    };

    let client = create_surrealdb_client(config_file, &branch_data_db_configuration).await?;
    Ok(client)
}

pub async fn create_branch_client(
    config_file: Option<&Path>,
    branch_name: &String,
    db_configuration: &SurrealdbConfiguration,
) -> Result<Surreal<Any>> {
    let branch_db_configuration = SurrealdbConfiguration {
        address: db_configuration.address.clone(),
        username: db_configuration.username.clone(),
        password: db_configuration.password.clone(),
        ns: Some(BRANCH_NS.to_owned()),
        db: Some(branch_name.to_owned()),
    };

    let client = create_surrealdb_client(config_file, &branch_db_configuration).await?;
    Ok(client)
}

pub async fn create_origin_branch_client(
    config_file: Option<&Path>,
    branch_name: &String,
    db_configuration: &SurrealdbConfiguration,
) -> Result<Surreal<Any>> {
    let branch_db_configuration = SurrealdbConfiguration {
        address: db_configuration.address.clone(),
        username: db_configuration.username.clone(),
        password: db_configuration.password.clone(),
        ns: Some(ORIGIN_BRANCH_NS.to_owned()),
        db: Some(branch_name.to_owned()),
    };

    let client = create_surrealdb_client(config_file, &branch_db_configuration).await?;
    Ok(client)
}

pub async fn create_main_branch_client(
    config_file: Option<&Path>,
    db_configuration: &SurrealdbConfiguration,
    branch: &Branch,
) -> Result<Surreal<Any>> {
    let main_branch_db_configuration = SurrealdbConfiguration {
        address: db_configuration.address.clone(),
        username: db_configuration.username.clone(),
        password: db_configuration.password.clone(),
        ns: Some(branch.from_ns.to_string()),
        db: Some(branch.from_db.to_string()),
    };

    let client = create_surrealdb_client(config_file, &main_branch_db_configuration).await?;
    Ok(client)
}

pub async fn get_branch_table(
    branching_feature_client: &Surreal<Any>,
    name: &String,
) -> Result<Option<Branch>> {
    if get_surrealdb_table_exists(branching_feature_client, BRANCH_TABLE).await? {
        let branch = branching_feature_client
            .select((BRANCH_TABLE, name.to_string()))
            .await?;
        Ok(branch)
    } else {
        Ok(None)
    }
}

pub async fn retrieve_existing_branch_names(
    branching_feature_client: &Surreal<Any>,
) -> Result<Vec<String>> {
    if get_surrealdb_table_exists(branching_feature_client, BRANCH_TABLE).await? {
        let existing_branch_names: Vec<String> = branching_feature_client
            .query(format!("SELECT VALUE name FROM {}", BRANCH_TABLE))
            .await?
            .take(0)?;

        Ok(existing_branch_names)
    } else {
        Ok(vec![])
    }
}

pub fn remove_dump_file(dump_file_path: &PathBuf) -> Result<()> {
    fs::remove_file(dump_file_path)?;
    Ok(())
}
