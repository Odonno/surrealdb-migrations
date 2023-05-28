use std::path::PathBuf;

use anyhow::{anyhow, Result};
use surrealdb::{engine::any::Any, Surreal};

use crate::{
    branch::constants::BRANCH_TABLE, config, input::SurrealdbConfiguration, io, models::Branch,
    surrealdb::create_surrealdb_client,
};

use super::{
    common::{create_branch_client, create_branching_feature_client, remove_dump_file},
    constants::DUMP_FILENAME,
};

pub async fn main(name: String, db_configuration: &SurrealdbConfiguration) -> Result<()> {
    let branching_feature_client = create_branching_feature_client(&db_configuration).await?;
    let branch: Option<Branch> = branching_feature_client
        .select((BRANCH_TABLE, name.to_string()))
        .await?;

    match branch {
        Some(branch) => {
            let folder_path = config::retrieve_folder_path();
            let dump_file_path = io::concat_path(&folder_path, DUMP_FILENAME);

            let branch_client = create_branch_client(&branch.name, &db_configuration).await?;
            branch_client.export(&dump_file_path).await?;

            let result =
                apply_changes_to_main_branch(db_configuration, &branch, &dump_file_path).await;

            match result {
                Ok(_) => {
                    remove_dump_file(&dump_file_path)?;
                    println!("Branch {} successfully merged", branch.name.to_string());

                    Ok(())
                }
                Err(error) => {
                    remove_dump_file(&dump_file_path)?;

                    Err(error)
                }
            }
        }
        None => Err(anyhow!("Branch {} does not exist", name)),
    }
}

async fn apply_changes_to_main_branch(
    db_configuration: &SurrealdbConfiguration,
    branch: &Branch,
    dump_file_path: &PathBuf,
) -> Result<()> {
    // Import the dump file into the main branch
    let main_branch_client = create_main_branch_client(db_configuration, branch).await?;
    main_branch_client.import(dump_file_path).await?;

    // Remove database created for this branch
    let branch_client = create_branch_client(&branch.name, &db_configuration).await?;
    branch_client
        .query(format!("REMOVE DATABASE ⟨{}⟩", branch.name.to_string()))
        .await?;

    // Remove branch from branches table
    let branch_data_client = create_branching_feature_client(&db_configuration).await?;
    let _record: Option<Branch> = branch_data_client
        .delete((BRANCH_TABLE, branch.name.to_string()))
        .await?;

    Ok(())
}

#[allow(deprecated)]
async fn create_main_branch_client(
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

    let client = create_surrealdb_client(&main_branch_db_configuration).await?;
    Ok(client)
}
