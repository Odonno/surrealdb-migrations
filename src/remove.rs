use anyhow::{anyhow, Context, Result};
use std::path::Path;

use crate::{
    config,
    constants::{DOWN_MIGRATIONS_DIR_NAME, MIGRATIONS_DIR_NAME},
    io::{self, SurqlFile},
};

pub fn main() -> Result<()> {
    let forward_migrations_files = io::extract_forward_migrations_files(None);

    if forward_migrations_files.is_empty() {
        return Err(anyhow!("No migration files left"));
    }

    let last_migration = forward_migrations_files
        .last()
        .context("Cannot get last migration")?;

    remove_migration_file(last_migration)?;
    remove_definition_file_if_exists(last_migration)?;
    remove_down_migration_file_if_exists(last_migration)?;

    let last_migration_display_name = get_migration_display_name(last_migration);

    println!(
        "Migration '{}' successfully removed",
        last_migration_display_name
    );

    Ok(())
}

fn remove_migration_file(last_migration: &SurqlFile) -> Result<()> {
    let folder_path = config::retrieve_folder_path();
    let migrations_path = io::concat_path(&folder_path, MIGRATIONS_DIR_NAME);

    let file_path = migrations_path.join(&last_migration.full_name);

    std::fs::remove_file(file_path)?;

    Ok(())
}

fn remove_definition_file_if_exists(last_migration: &SurqlFile) -> Result<()> {
    let folder_path = config::retrieve_folder_path();
    let migrations_path = io::concat_path(&folder_path, MIGRATIONS_DIR_NAME);

    let migration_definition_file_path = Path::new(&migrations_path)
        .join("definitions")
        .join(format!("{}.json", last_migration.name));

    if migration_definition_file_path.exists() {
        std::fs::remove_file(migration_definition_file_path)?;
    }

    Ok(())
}

fn remove_down_migration_file_if_exists(last_migration: &SurqlFile) -> Result<()> {
    remove_nested_down_migration_file_if_exists(last_migration)?;
    remove_inlined_down_migration_file_if_exists(last_migration)?;

    Ok(())
}

fn remove_nested_down_migration_file_if_exists(last_migration: &SurqlFile) -> Result<()> {
    let folder_path = config::retrieve_folder_path();
    let migrations_path = io::concat_path(&folder_path, MIGRATIONS_DIR_NAME);

    let down_migration_file_path = Path::new(&migrations_path)
        .join(DOWN_MIGRATIONS_DIR_NAME)
        .join(&last_migration.full_name);

    if down_migration_file_path.exists() {
        std::fs::remove_file(down_migration_file_path)?;
    }

    Ok(())
}

fn remove_inlined_down_migration_file_if_exists(last_migration: &SurqlFile) -> Result<()> {
    let folder_path = config::retrieve_folder_path();
    let migrations_path = io::concat_path(&folder_path, MIGRATIONS_DIR_NAME);

    let inlined_down_migration_file_path =
        Path::new(&migrations_path).join(format!("{}.down.surql", last_migration.name));

    if inlined_down_migration_file_path.exists() {
        std::fs::remove_file(inlined_down_migration_file_path)?;
    }

    Ok(())
}

fn get_migration_display_name(migration_file: &SurqlFile) -> String {
    migration_file
        .name
        .split('_')
        .skip(2)
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .join("_")
}
