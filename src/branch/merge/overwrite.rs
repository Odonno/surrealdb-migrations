use anyhow::Result;
use std::path::PathBuf;

use crate::{
    branch::{
        common::{
            create_branch_client, create_branching_feature_client, create_main_branch_client,
            create_origin_branch_client, remove_dump_file,
        },
        constants::{BRANCH_TABLE, DUMP_FILENAME},
    },
    config,
    input::SurrealdbConfiguration,
    io,
    models::Branch,
};

pub async fn main(branch: Branch, db_configuration: &SurrealdbConfiguration) -> Result<()> {
    let folder_path = config::retrieve_folder_path();
    let dump_file_path = io::concat_path(&folder_path, DUMP_FILENAME);

    let branch_client = create_branch_client(&branch.name, db_configuration).await?;
    branch_client.export(&dump_file_path).await?;

    let result = apply_changes_to_main_branch(db_configuration, &branch, &dump_file_path).await;

    match result {
        Ok(_) => {
            remove_dump_file(&dump_file_path)?;
            println!("Branch {} successfully merged", branch.name);

            Ok(())
        }
        Err(error) => {
            remove_dump_file(&dump_file_path)?;

            Err(error)
        }
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

    // Remove databases created for this branch
    let client = create_branch_client(&branch.name, db_configuration).await?;
    client
        .query(format!("REMOVE DATABASE ⟨{}⟩", branch.name))
        .await?;

    let client = create_origin_branch_client(&branch.name, db_configuration).await?;
    client
        .query(format!("REMOVE DATABASE ⟨{}⟩", branch.name))
        .await?;

    // Remove branch from branches table
    let branch_data_client = create_branching_feature_client(db_configuration).await?;
    branch_data_client
        .delete::<Option<Branch>>((BRANCH_TABLE, branch.name.to_string()))
        .await?;

    Ok(())
}
