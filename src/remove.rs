use color_eyre::eyre::{eyre, ContextCompat, Result};
use std::{collections::HashSet, path::Path};

use crate::{
    common::get_migration_display_name,
    config,
    constants::{
        ALL_TAGS, DOWN_MIGRATIONS_DIR_NAME, DOWN_SURQL_FILE_EXTENSION, MIGRATIONS_DIR_NAME,
    },
    file::SurqlFile,
    io::{self},
    models::MigrationDirection,
};

pub fn main(config_file: Option<&Path>) -> Result<()> {
    let tags = HashSet::from([ALL_TAGS.into()]);
    let exclude_tags = HashSet::new();
    let forward_migrations_files = io::extract_migrations_files(
        config_file,
        None,
        MigrationDirection::Forward,
        &tags,
        &exclude_tags,
    );

    if forward_migrations_files.is_empty() {
        return Err(eyre!("No migration files left"));
    }

    let last_migration = forward_migrations_files
        .last()
        .context("Cannot get last migration")?;

    remove_migration_file(config_file, last_migration)?;
    remove_definition_file_if_exists(config_file, last_migration)?;
    remove_down_migration_file_if_exists(config_file, last_migration)?;

    let last_migration_display_name = get_migration_display_name(&last_migration.name);

    println!(
        "Migration '{}' successfully removed",
        last_migration_display_name
    );

    Ok(())
}

fn remove_migration_file(config_file: Option<&Path>, last_migration: &SurqlFile) -> Result<()> {
    let folder_path = config::retrieve_folder_path(config_file);
    let migrations_path = io::concat_path(&folder_path, MIGRATIONS_DIR_NAME);

    let file_path = migrations_path.join(&last_migration.full_name);

    std::fs::remove_file(file_path)?;

    Ok(())
}

fn remove_definition_file_if_exists(
    config_file: Option<&Path>,
    last_migration: &SurqlFile,
) -> Result<()> {
    let folder_path = config::retrieve_folder_path(config_file);
    let migrations_path = io::concat_path(&folder_path, MIGRATIONS_DIR_NAME);

    let migration_definition_file_path = Path::new(&migrations_path)
        .join("definitions")
        .join(format!("{}.json", last_migration.name));

    if migration_definition_file_path.exists() {
        std::fs::remove_file(migration_definition_file_path)?;
    }

    Ok(())
}

fn remove_down_migration_file_if_exists(
    config_file: Option<&Path>,
    last_migration: &SurqlFile,
) -> Result<()> {
    remove_nested_down_migration_file_if_exists(config_file, last_migration)?;
    remove_inlined_down_migration_file_if_exists(config_file, last_migration)?;

    Ok(())
}

fn remove_nested_down_migration_file_if_exists(
    config_file: Option<&Path>,
    last_migration: &SurqlFile,
) -> Result<()> {
    let folder_path = config::retrieve_folder_path(config_file);
    let migrations_path = io::concat_path(&folder_path, MIGRATIONS_DIR_NAME);

    let down_migration_file_path = Path::new(&migrations_path)
        .join(DOWN_MIGRATIONS_DIR_NAME)
        .join(&last_migration.full_name);

    if down_migration_file_path.exists() {
        std::fs::remove_file(down_migration_file_path)?;
    }

    Ok(())
}

fn remove_inlined_down_migration_file_if_exists(
    config_file: Option<&Path>,
    last_migration: &SurqlFile,
) -> Result<()> {
    let folder_path = config::retrieve_folder_path(config_file);
    let migrations_path = io::concat_path(&folder_path, MIGRATIONS_DIR_NAME);

    let inlined_down_migration_file_path = Path::new(&migrations_path).join(format!(
        "{}{}",
        last_migration.name, DOWN_SURQL_FILE_EXTENSION
    ));

    if inlined_down_migration_file_path.exists() {
        std::fs::remove_file(inlined_down_migration_file_path)?;
    }

    Ok(())
}
