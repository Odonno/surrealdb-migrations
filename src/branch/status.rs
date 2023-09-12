use chrono::{DateTime, Utc};
use chrono_human_duration::ChronoHumanDuration;
use color_eyre::eyre::{eyre, Result};
use std::path::Path;

use crate::{
    branch::{
        common::create_branching_feature_client,
        constants::{BRANCH_NS, BRANCH_TABLE},
    },
    input::SurrealdbConfiguration,
    models::Branch,
};

pub struct BranchStatusArgs<'a> {
    pub name: String,
    pub config_file: Option<&'a Path>,
}

pub async fn main(args: BranchStatusArgs<'_>) -> Result<()> {
    let BranchStatusArgs { name, config_file } = args;

    let db_configuration = SurrealdbConfiguration::default();
    let branching_feature_client =
        create_branching_feature_client(config_file, &db_configuration).await?;
    let branch: Option<Branch> = branching_feature_client
        .select((BRANCH_TABLE, name.to_string()))
        .await?;

    match branch {
        Some(branch) => {
            let now = Utc::now();

            let parsed_created_at = DateTime::parse_from_rfc3339(&branch.created_at)?;

            let display_created_at = parsed_created_at.format("%c");

            let since = now.signed_duration_since(parsed_created_at);
            let since = since.format_human().to_string();

            println!("## Branch status ##");
            println!("Name: {}", branch.name);
            println!("Namespace: {}", BRANCH_NS);
            println!("Database: {}", branch.name);
            println!("Created at: {} ({})", display_created_at, since);
            println!();
            println!("## Origin Branch ##");
            println!("Namespace: {}", branch.from_ns);
            println!("Database: {}", branch.from_db);

            Ok(())
        }
        None => Err(eyre!("Branch {} does not exist", name)),
    }
}
