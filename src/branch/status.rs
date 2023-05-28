use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use chrono_human_duration::ChronoHumanDuration;

use crate::{
    branch::{
        common::create_branching_feature_client,
        constants::{BRANCH_NS, BRANCH_TABLE},
    },
    input::SurrealdbConfiguration,
    models::Branch,
};

pub async fn main(name: String) -> Result<()> {
    let db_configuration = SurrealdbConfiguration::default();
    let branching_feature_client = create_branching_feature_client(&db_configuration).await?;
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
        None => Err(anyhow!("Branch {} does not exist", name)),
    }
}
