use color_eyre::eyre::{eyre, Result};
use std::path::Path;

use crate::{cli::BranchMergeMode, input::SurrealdbConfiguration, models::Branch};

use self::overwrite::MergeOverwriteBranchArgs;

use super::common::{create_branching_feature_client, get_branch_table};

mod overwrite;

pub struct MergeBranchArgs<'a> {
    pub name: String,
    pub mode: BranchMergeMode,
    pub db_configuration: SurrealdbConfiguration,
    pub config_file: Option<&'a Path>,
}

pub async fn main(args: MergeBranchArgs<'_>) -> Result<()> {
    let MergeBranchArgs {
        name,
        mode,
        db_configuration,
        config_file,
    } = args;

    let branching_feature_client =
        create_branching_feature_client(config_file, &db_configuration).await?;

    let branch: Option<Branch> = get_branch_table(&branching_feature_client, &name).await?;

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
