use color_eyre::eyre::{eyre, Result};

use crate::{
    branch::constants::BRANCH_TABLE, cli::BranchMergeMode, input::SurrealdbConfiguration,
    models::Branch,
};

use self::overwrite::MergeOverwriteBranchArgs;

use super::common::create_branching_feature_client;

mod overwrite;

pub struct MergeBranchArgs<'a> {
    pub name: String,
    pub mode: BranchMergeMode,
    pub db_configuration: &'a SurrealdbConfiguration,
    pub config_file: Option<&'a str>,
}

pub async fn main(args: MergeBranchArgs<'_>) -> Result<()> {
    let MergeBranchArgs {
        name,
        mode,
        db_configuration,
        config_file,
    } = args;

    let branching_feature_client =
        create_branching_feature_client(config_file, db_configuration).await?;
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
            BranchMergeMode::Overwrite => {
                let args = MergeOverwriteBranchArgs {
                    branch,
                    db_configuration,
                    config_file,
                };
                overwrite::main(args).await
            }
        },
        None => Err(eyre!("Branch {} does not exist", name)),
    }
}
