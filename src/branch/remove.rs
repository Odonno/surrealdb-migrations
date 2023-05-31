use anyhow::{anyhow, Result};

use crate::{
    branch::{
        common::{
            create_branch_client, create_branching_feature_client, create_origin_branch_client,
            retrieve_existing_branch_names,
        },
        constants::{BRANCH_NS, BRANCH_TABLE},
    },
    input::SurrealdbConfiguration,
    models::Branch,
};

pub struct RemoveBranchArgs<'a> {
    pub name: String,
    pub db_configuration: &'a SurrealdbConfiguration,
    pub config_file: Option<&'a str>,
}

pub async fn main(args: RemoveBranchArgs<'_>) -> Result<()> {
    let RemoveBranchArgs {
        name,
        db_configuration,
        config_file,
    } = args;

    let branching_feature_client =
        create_branching_feature_client(config_file, db_configuration).await?;

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

    // Remove databases created for this branch
    let client = create_branch_client(config_file, &name, db_configuration).await?;
    client.query(format!("REMOVE DATABASE ⟨{}⟩", name)).await?;

    let client = create_origin_branch_client(config_file, &name, db_configuration).await?;
    client.query(format!("REMOVE DATABASE ⟨{}⟩", name)).await?;

    // Remove branch from branches table
    branching_feature_client
        .delete::<Option<Branch>>((BRANCH_TABLE, name.to_string()))
        .await?;

    println!("Branch {} successfully removed", name);

    Ok(())
}
