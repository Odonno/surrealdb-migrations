pub mod args;

pub use args::ListArgs;
use chrono::{DateTime, Utc};
use chrono_human_duration::ChronoHumanDuration;
use cli_table::{format::Border, Cell, ColorChoice, Style, Table};
use color_eyre::eyre::Result;

use crate::{
    common::get_migration_display_name, constants::SURQL_FILE_EXTENSION,
    runbin::surrealdb::create_surrealdb_client,
    surrealdb::list_script_migration_ordered_by_execution_date,
};

pub async fn main(args: ListArgs<'_>) -> Result<()> {
    let ListArgs {
        db_configuration,
        no_color,
        config_file,
    } = args;

    let client = create_surrealdb_client(config_file, &db_configuration).await?;

    let migrations_applied = list_script_migration_ordered_by_execution_date(&client).await?;

    if migrations_applied.is_empty() {
        println!("No migrations applied yet!");
    } else {
        let now = Utc::now();

        let rows = migrations_applied
            .iter()
            .map(|m| {
                let display_name = get_migration_display_name(&m.script_name);

                let since = match DateTime::parse_from_rfc3339(&m.executed_at) {
                    Ok(executed_at) => {
                        let since = now.signed_duration_since(executed_at);
                        since.format_human().to_string()
                    }
                    Err(_) => "N/A".to_string(),
                };

                let file_name = m.script_name.clone() + SURQL_FILE_EXTENSION;

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
