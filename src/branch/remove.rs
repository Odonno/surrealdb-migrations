use anyhow::{anyhow, Result};

use crate::{
    branch::{
        common::{
            create_branch_client, create_branching_feature_client, retrieve_existing_branch_names,
        },
        constants::{BRANCH_NS, BRANCH_TABLE},
    },
    input::SurrealdbConfiguration,
    models::Branch,
};

pub async fn main(name: String, db_configuration: &SurrealdbConfiguration) -> Result<()> {
    let branching_feature_client = create_branching_feature_client(db_configuration).await?;

    // Check if branch really exists
    let existing_branch_names = retrieve_existing_branch_names(&branching_feature_client).await?;

    if !existing_branch_names.contains(&name) {
        return Err(anyhow!("Branch {} does not exist", name));
    }

    // Prevent branch to be removed if used by another branch
    let number_of_dependent_branches: Option<u32> = branching_feature_client
        .query("SELECT VALUE count() FROM branch WHERE from_ns = $ns AND from_db = $db")
        .bind(("ns", BRANCH_NS))
        .bind(("db", name.to_string()))
        .await?
        .take(0)?;
    let number_of_dependent_branches = number_of_dependent_branches.unwrap_or(0);

    if number_of_dependent_branches > 0 {
        return Err(anyhow!("Branch {} is used by another branch", name));
    }

    // Remove database created for this branch
    let client = create_branch_client(&name, &db_configuration).await?;
    client
        .query(format!("REMOVE DATABASE ⟨{}⟩", name.to_string()))
        .await?;

    // Remove branch from branches table
    let _record: Option<Branch> = branching_feature_client
        .delete((BRANCH_TABLE, name.to_string()))
        .await?;

    println!("Branch {} successfully removed", name.to_string());

    Ok(())
}
