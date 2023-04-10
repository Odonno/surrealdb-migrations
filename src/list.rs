use chrono::{DateTime, Utc};
use chrono_human_duration::ChronoHumanDuration;
use cli_table::{format::Border, Cell, ColorChoice, Style, Table};
use std::process;
use surrealdb::{engine::remote::ws::Ws, opt::auth::Root, Surreal};

use crate::{config, models::ScriptMigration};

pub async fn main(
    url: Option<String>,
    ns: Option<String>,
    db: Option<String>,
    username: Option<String>,
    password: Option<String>,
    no_color: bool,
) {
    let db_config = config::retrieve_db_config();

    let url = url.or(db_config.url).unwrap_or("localhost:8000".to_owned());

    let connection = Surreal::new::<Ws>(url.to_owned()).await;

    if let Err(error) = connection {
        eprintln!("{}", error);
        process::exit(1);
    }

    let client = connection.unwrap();

    let username = username.or(db_config.username).unwrap_or("root".to_owned());
    let password = password.or(db_config.password).unwrap_or("root".to_owned());

    client
        .signin(Root {
            username: &username,
            password: &password,
        })
        .await
        .unwrap();

    let ns = ns.or(db_config.ns).unwrap_or("test".to_owned());
    let db = db.or(db_config.db).unwrap_or("test".to_owned());

    client
        .use_ns(ns.to_owned())
        .use_db(db.to_owned())
        .await
        .unwrap();

    let response = client.select("script_migration").await;

    if let Err(error) = response {
        eprintln!("{}", error);
        process::exit(1);
    }

    let mut migrations_applied: Vec<ScriptMigration> = response.unwrap();
    migrations_applied.sort_by_key(|m| m.executed_at.clone());

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

                let executed_at = DateTime::parse_from_rfc3339(&m.executed_at).unwrap();
                let since = now.signed_duration_since(executed_at);
                let since = since.format_human().to_string();

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

        let table_display = table.display().unwrap();

        println!("{}", table_display);
    }
}
