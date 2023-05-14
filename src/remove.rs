use anyhow::{anyhow, Context, Result};
use fs_extra::dir::{DirEntryAttr, DirEntryValue};
use std::{collections::HashSet, path::Path};

use crate::{config, constants::MIGRATIONS_DIR_NAME};

pub fn main() -> Result<()> {
    let folder_path = config::retrieve_folder_path();

    let migrations_path = match folder_path.to_owned() {
        Some(folder_path) => Path::new(&folder_path).join(MIGRATIONS_DIR_NAME),
        None => Path::new(MIGRATIONS_DIR_NAME).to_path_buf(),
    };

    let mut config = HashSet::new();
    config.insert(DirEntryAttr::Name);
    config.insert(DirEntryAttr::Path);
    config.insert(DirEntryAttr::FullName);
    config.insert(DirEntryAttr::IsFile);

    let migrations_files = fs_extra::dir::ls(&migrations_path, &config)?;
    let migrations_files = migrations_files
        .items
        .iter()
        .filter(
            |migration_file| match migration_file.get(&DirEntryAttr::IsFile) {
                Some(DirEntryValue::Boolean(is_file)) => *is_file,
                _ => false,
            },
        )
        .collect::<Vec<_>>();

    if migrations_files.is_empty() {
        return Err(anyhow!("No migration files left"));
    }

    let mut sorted_migrations_files = migrations_files;
    sorted_migrations_files.sort_by(|a, b| {
        let a = a.get(&DirEntryAttr::Name);
        let b = b.get(&DirEntryAttr::Name);

        let a = match a {
            Some(DirEntryValue::String(a)) => Some(a),
            _ => None,
        };

        let b = match b {
            Some(DirEntryValue::String(b)) => Some(b),
            _ => None,
        };

        a.cmp(&b)
    });

    let last_migration = sorted_migrations_files
        .last()
        .context("Cannot get last migration")?;

    let last_migration_filename = match last_migration.get(&DirEntryAttr::Name) {
        Some(DirEntryValue::String(last_migration_filename)) => Ok(last_migration_filename),
        _ => Err(anyhow!("Cannot get name to migration files")),
    }?;

    let last_migration_fullname = match last_migration.get(&DirEntryAttr::FullName) {
        Some(DirEntryValue::String(last_migration_filename)) => Ok(last_migration_filename),
        _ => Err(anyhow!("Cannot get name to migration files")),
    }?;

    let last_migration_display_name = last_migration_filename
        .split("_")
        .skip(2)
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .join("_");

    // Remove migration file
    let migration_file = migrations_path.join(last_migration_fullname);
    std::fs::remove_file(migration_file)?;

    // Remove definition file if exists
    let migration_definition_file_path = Path::new(&migrations_path)
        .join("definitions")
        .join(format!("{}.json", last_migration_filename));

    if migration_definition_file_path.exists() {
        std::fs::remove_file(migration_definition_file_path)?;
    }

    // Remove down migration file if exists
    let down_migration_file_path = Path::new(&migrations_path)
        .join("down")
        .join(last_migration_fullname);

    if down_migration_file_path.exists() {
        std::fs::remove_file(down_migration_file_path)?;
    }

    // Remove inlined down migration file if exists
    let inlined_down_migration_file_path =
        Path::new(&migrations_path).join(format!("{}.down.surql", last_migration_filename));

    if inlined_down_migration_file_path.exists() {
        std::fs::remove_file(inlined_down_migration_file_path)?;
    }

    println!(
        "Migration '{}' successfully removed",
        last_migration_display_name
    );

    Ok(())
}
