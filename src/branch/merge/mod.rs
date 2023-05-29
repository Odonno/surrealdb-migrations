use anyhow::{anyhow, Result};

use crate::{
    branch::constants::BRANCH_TABLE, cli::BranchMergeMode, input::SurrealdbConfiguration,
    models::Branch,
};

use super::common::create_branching_feature_client;

mod overwrite;

pub async fn main(
    name: String,
    mode: BranchMergeMode,
    db_configuration: &SurrealdbConfiguration,
) -> Result<()> {
    let branching_feature_client = create_branching_feature_client(db_configuration).await?;
    let branch: Option<Branch> = branching_feature_client
        .select((BRANCH_TABLE, name.to_string()))
        .await?;

    match branch {
        Some(branch) => match mode {
            BranchMergeMode::SchemaOnly => {
                todo!()
            }
            BranchMergeMode::All => {
                todo!()
            }
            BranchMergeMode::Overwrite => overwrite::main(branch, db_configuration).await,
        },
        None => Err(anyhow!("Branch {} does not exist", name)),
    }
}
