pub mod args;

pub use args::StatusArgs;
use color_eyre::eyre::{eyre, Result};
use owo_colors::{self, OwoColorize, Stream::Stdout};
use std::collections::HashSet;

use crate::{
    constants::{ALL_TAGS, SCRIPT_MIGRATION_TABLE_NAME},
    io,
    models::MigrationDirection,
    runbin::surrealdb::create_surrealdb_client,
    surrealdb::{
        get_surrealdb_table_definition, get_surrealdb_table_definitions,
        list_script_migration_ordered_by_execution_date,
    },
};

pub async fn main(args: StatusArgs<'_>) -> Result<()> {
    let StatusArgs {
        db_configuration,
        no_color,
        config_file,
    } = args;

    if !no_color {
        owo_colors::set_override(false);
    }

    let client = create_surrealdb_client(config_file, &db_configuration).await?;

    let table_definitions = get_surrealdb_table_definitions(&client).await?;

    if !table_definitions.contains_key(SCRIPT_MIGRATION_TABLE_NAME) {
        return Err(eyre!("The table '{}' does not exist. Make sure to apply the migrations once before running this command.", SCRIPT_MIGRATION_TABLE_NAME));
    }

    let script_migration_table_definition =
        get_surrealdb_table_definition(&client, SCRIPT_MIGRATION_TABLE_NAME).await?;

    let tags = HashSet::from([ALL_TAGS.into()]);

    let schemas_files = io::extract_schemas_files(config_file, None, &tags)
        .ok()
        .unwrap_or_default();
    let events_files = io::extract_events_files(config_file, None, &tags)
        .ok()
        .unwrap_or_default();

    let schema_definitions = io::concat_files_content(&schemas_files);
    let event_definitions = io::concat_files_content(&events_files);

    let forward_migrations_files =
        io::extract_migrations_files(config_file, None, MigrationDirection::Forward, &tags);

    let use_traditional_approach = schema_definitions.is_empty()
        && event_definitions.is_empty()
        && !forward_migrations_files.is_empty();

    let use_migration_definitions = !use_traditional_approach;

    let supports_checksum = script_migration_table_definition
        .fields
        .contains_key("checksum");

    let migrations_applied = list_script_migration_ordered_by_execution_date(&client).await?;

    let names_of_migrations_applied = migrations_applied
        .iter()
        .map(|m| m.script_name.to_string())
        .collect::<HashSet<_>>();

    let names_of_migrations_to_apply = forward_migrations_files
        .iter()
        .map(|f| f.name.to_string())
        .collect::<HashSet<_>>();

    let number_of_migrations_applied = migrations_applied.len();

    let left_migrations_to_apply = names_of_migrations_to_apply
        .difference(&names_of_migrations_applied)
        .count();

    let missing_migrations_files =
        names_of_migrations_applied.difference(&names_of_migrations_to_apply);
    let missing_migrations_files_count = missing_migrations_files.clone().count();

    print!(
        "Total of migrations applied: {}",
        (if number_of_migrations_applied > 0 {
            number_of_migrations_applied.to_string()
        } else {
            String::from("none")
        })
        .if_supports_color(Stdout, |text| text.yellow())
    );
    if left_migrations_to_apply > 0 {
        print!(" ");
        let text = format!("↓{}", left_migrations_to_apply);
        print!("{}", text.if_supports_color(Stdout, |text| text.red()));
    }
    if missing_migrations_files_count > 0 {
        print!(" ");
        let text = format!("?{}", missing_migrations_files_count);
        print!(
            "{}",
            text.if_supports_color(Stdout, |text| text.bright_red())
        );
    }

    println!();
    println!();

    println!("Capabilities");
    println!(
        "- Definition files: {}",
        get_feature_check_str(use_migration_definitions)
    );
    println!("- Checksum: {}", get_feature_check_str(supports_checksum));

    let is_up_to_date = left_migrations_to_apply == 0 && missing_migrations_files_count == 0;
    if is_up_to_date {
        println!();
        println!("✅ Database migrations are up to date.");
    }

    if missing_migrations_files_count > 0 {
        println!();
        println!("❓ The following files seems to be missing:");

        for filename in missing_migrations_files {
            println!("- {}", filename);
        }
    }

    Ok(())
}

fn get_feature_check_str(enabled: bool) -> &'static str {
    if enabled {
        "✅"
    } else {
        "❌"
    }
}
