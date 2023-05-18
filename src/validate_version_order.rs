use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use ::surrealdb::{engine::any::Any, Surreal};
use anyhow::{anyhow, Context, Result};
use fs_extra::dir::{DirEntryAttr, DirEntryValue, LsResult};

use crate::{config, constants::MIGRATIONS_DIR_NAME, models::ScriptMigration, surrealdb};

pub async fn main(client: &Surreal<Any>) -> Result<()> {
    let migrations_applied =
        surrealdb::list_script_migration_ordered_by_execution_date(&client).await?;

    let mut config = HashSet::new();
    config.insert(DirEntryAttr::Name);
    config.insert(DirEntryAttr::Path);
    config.insert(DirEntryAttr::IsFile);

    let folder_path = config::retrieve_folder_path();
    let migrations_dir_path = concat_path(&folder_path, MIGRATIONS_DIR_NAME);

    let migrations_files = fs_extra::dir::ls(migrations_dir_path, &config)?;

    let migrations_not_applied = get_sorted_migrations_files(&migrations_files)
        .into_iter()
        .filter(|migration_file| {
            is_migration_file_already_applied(migration_file, &migrations_applied).unwrap_or(false)
        })
        .collect::<Vec<_>>();

    let last_migration_applied = migrations_applied.last();

    let migrations_not_applied_before_last_applied =
        if let Some(last_migration_applied) = last_migration_applied {
            migrations_not_applied
                .iter()
                .filter(|migration_file| {
                    is_migration_file_before_last_applied(migration_file, &last_migration_applied)
                        .unwrap_or(false)
                })
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };

    if migrations_not_applied_before_last_applied.len() > 0 {
        let migration_names = migrations_not_applied_before_last_applied
            .iter()
            .map(|migration_file| get_migration_file_name(migration_file).unwrap_or("".to_string()))
            .collect::<Vec<_>>();

        Err(anyhow!(
            "The following migrations have not been applied: {}",
            migration_names.join(", ")
        ))
    } else {
        Ok(())
    }
}

fn concat_path(folder_path: &Option<String>, dir_name: &str) -> PathBuf {
    match folder_path.to_owned() {
        Some(folder_path) => Path::new(&folder_path).join(dir_name),
        None => Path::new(dir_name).to_path_buf(),
    }
}

fn get_sorted_migrations_files(
    migrations_files: &LsResult,
) -> Vec<&HashMap<DirEntryAttr, DirEntryValue>> {
    let mut sorted_migrations_files = migrations_files.items.iter().collect::<Vec<_>>();
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

    sorted_migrations_files
}

fn is_migration_file_already_applied(
    migration_file: &&std::collections::HashMap<DirEntryAttr, DirEntryValue>,
    migrations_applied: &Vec<ScriptMigration>,
) -> Result<bool> {
    let is_file = migration_file
        .get(&DirEntryAttr::IsFile)
        .context("Cannot detect if the migration file is a file or a folder")?;
    let is_file = match is_file {
        DirEntryValue::Boolean(is_file) => Some(is_file),
        _ => None,
    };
    let is_file = is_file.context("Cannot detect if the migration file is a file or a folder")?;

    if !is_file {
        return Ok(false);
    }

    let name = migration_file
        .get(&DirEntryAttr::Name)
        .context("Cannot get name of the migration file")?;
    let name = match name {
        DirEntryValue::String(name) => Some(name),
        _ => None,
    };
    let name = name.context("Cannot get name of the migration file")?;

    let has_already_been_applied = migrations_applied
        .iter()
        .any(|migration_applied| &migration_applied.script_name == name);

    if has_already_been_applied {
        return Ok(false);
    }

    return Ok(true);
}

fn is_migration_file_before_last_applied(
    migration_file: &&HashMap<DirEntryAttr, DirEntryValue>,
    last_migration_applied: &ScriptMigration,
) -> Result<bool> {
    let is_file = migration_file
        .get(&DirEntryAttr::IsFile)
        .context("Cannot detect if the migration file is a file or a folder")?;
    let is_file = match is_file {
        DirEntryValue::Boolean(is_file) => Some(is_file),
        _ => None,
    };
    let is_file = is_file.context("Cannot detect if the migration file is a file or a folder")?;

    if !is_file {
        return Ok(false);
    }

    let name = migration_file
        .get(&DirEntryAttr::Name)
        .context("Cannot get name of the migration file")?;
    let name = match name {
        DirEntryValue::String(name) => Some(name),
        _ => None,
    };
    let name = name.context("Cannot get name of the migration file")?;

    Ok(name < &last_migration_applied.script_name)
}

fn get_migration_file_name(
    migration_file: &&HashMap<DirEntryAttr, DirEntryValue>,
) -> Result<String> {
    let name = migration_file
        .get(&DirEntryAttr::Name)
        .context("Cannot get name of the migration file")?;
    let name = match name {
        DirEntryValue::String(name) => Some(name),
        _ => None,
    };
    let name = name.context("Cannot get name of the migration file")?;

    Ok(name.to_string())
}
