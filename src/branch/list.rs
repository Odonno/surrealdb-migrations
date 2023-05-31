use anyhow::Result;
use chrono::{DateTime, Utc};
use chrono_human_duration::ChronoHumanDuration;
use cli_table::{format::Border, Cell, ColorChoice, Style, Table};

use crate::{
    branch::{
        common::create_branching_feature_client,
        constants::{BRANCH_NS, BRANCH_TABLE},
    },
    input::SurrealdbConfiguration,
    models::Branch,
};

pub struct ListBranchArgs<'a> {
    pub db_configuration: &'a SurrealdbConfiguration,
    pub no_color: bool,
    pub config_file: Option<&'a str>,
}

pub async fn main(args: ListBranchArgs<'_>) -> Result<()> {
    let ListBranchArgs {
        db_configuration,
        no_color,
        config_file,
    } = args;

    let branching_feature_client =
        create_branching_feature_client(config_file, db_configuration).await?;
    let existing_branches: Vec<Branch> = branching_feature_client.select(BRANCH_TABLE).await?;

    if existing_branches.is_empty() {
        println!("There are no branch yet!");
    } else {
        let now = Utc::now();

        let rows = existing_branches
            .iter()
            .map(|b| {
                let since = match DateTime::parse_from_rfc3339(&b.created_at) {
                    Ok(created_at) => {
                        let since = now.signed_duration_since(created_at);
                        since.format_human().to_string()
                    }
                    Err(_) => "N/A".to_string(),
                };

                vec![
                    b.name.to_string().cell(),
                    b.from_ns.to_string().cell(),
                    b.from_db.to_string().cell(),
                    BRANCH_NS.cell(),
                    b.name.to_string().cell(),
                    since.cell(),
                ]
            })
            .collect::<Vec<_>>();

        let color_choice = if no_color {
            ColorChoice::Never
        } else {
            ColorChoice::Auto
        };

        let table = rows
            .table()
            .title(vec![
                "Name".cell().bold(true),
                "NS (main)".cell().bold(true),
                "DB (main)".cell().bold(true),
                "NS (branch)".cell().bold(true),
                "DB (branch)".cell().bold(true),
                "Created at".cell().bold(true),
            ])
            .color_choice(color_choice)
            .border(Border::builder().build());

        let table_display = table.display()?;

        println!("{}", table_display);
    }

    Ok(())
}
