use anyhow::Result;
use chrono::{DateTime, Utc};
use chrono_human_duration::ChronoHumanDuration;
use cli_table::{format::Border, Cell, ColorChoice, Style, Table};

use crate::surrealdb;

pub async fn main(
    url: Option<String>,
    ns: Option<String>,
    db: Option<String>,
    username: Option<String>,
    password: Option<String>,
    no_color: bool,
) -> Result<()> {
    let client = surrealdb::create_surrealdb_client(url, ns, db, username, password).await?;

    let migrations_applied =
        surrealdb::list_script_migration_ordered_by_execution_date(&client).await?;

    if migrations_applied.is_empty() {
        println!("No migrations applied yet!");
    } else {
        let now = Utc::now();

        let rows = migrations_applied
            .iter()
            .map(|m| {
                let display_name = m
                    .script_name
                    .split("_")
                    .skip(2)
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
                    .join("_");

                let since = match DateTime::parse_from_rfc3339(&m.executed_at) {
                    Ok(executed_at) => {
                        let since = now.signed_duration_since(executed_at);
                        since.format_human().to_string()
                    }
                    Err(_) => "N/A".to_string(),
                };

                let file_name = m.script_name.clone() + ".surql";

                vec![display_name.cell(), since.cell(), file_name.cell()]
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
                "Executed at".cell().bold(true),
                "File name".cell().bold(true),
            ])
            .color_choice(color_choice)
            .border(Border::builder().build());

        let table_display = table.display()?;

        println!("{}", table_display);
    }

    Ok(())
}
