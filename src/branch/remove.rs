use anyhow::{anyhow, Result};

use crate::{
    branch::{
        common::{create_branch_client, create_branch_data_client, retrieve_existing_branch_names},
        constants::BRANCH_TABLE,
    },
    input::SurrealdbConfiguration,
    models::Branch,
};

pub async fn main(name: String, db_configuration: &SurrealdbConfiguration) -> Result<()> {
    let branch_data_client = create_branch_data_client(db_configuration).await?;

    // Check if branch really exists
    let existing_branch_names = retrieve_existing_branch_names(&branch_data_client).await?;

    if !existing_branch_names.contains(&name) {
        return Err(anyhow!("Branch {} does not exist", name));
    }

    // Remove database created for this branch
    let client = create_branch_client(&name, &db_configuration).await?;
    client
        .query(format!("REMOVE DATABASE ⟨{}⟩", name.to_string()))
        .await?;

    // Remove branch from branches table
    let _record: Option<Branch> = branch_data_client
        .delete((BRANCH_TABLE, name.to_string()))
        .await?;

    println!("Branch {} successfully removed", name.to_string());

    Ok(())
}
