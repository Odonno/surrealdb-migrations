use ::surrealdb::{engine::any::Any, Surreal};
use anyhow::{anyhow, Context, Result};
use fs_extra::dir::{DirEntryAttr, DirEntryValue};
use include_dir::Dir;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use crate::{config, constants::MIGRATIONS_DIR_NAME, models::ScriptMigration, surrealdb};

pub struct ValidateVersionArgs<'a> {
    pub db: &'a Surreal<Any>,
    pub dir: Option<&'a Dir<'a>>,
}

#[derive(Debug)]
struct MigrationFile {
    name: String,
    is_file: bool,
}

pub async fn main<'a>(args: ValidateVersionArgs<'a>) -> Result<()> {
    let ValidateVersionArgs { db: client, dir } = args;

    let migrations_applied =
        surrealdb::list_script_migration_ordered_by_execution_date(&client).await?;

    // TODO : Filter .down.surql files
    let migrations_files = match dir {
        Some(dir) => {
            let migrations_dir = dir
                .get_dir(MIGRATIONS_DIR_NAME)
                .context("Migrations directory not found")?;

            migrations_dir
                .files()
                .filter_map(|f| {
                    let name = f.path().file_stem();
                    let name = match name {
                        Some(name)
                            if name.to_str().and_then(|n| Some(n.ends_with(".down")))
                                == Some(true) =>
                        {
                            Path::new(name).file_stem()
                        }
                        Some(name) => Some(name),
                        None => None,
                    };
                    let name = name
                        .and_then(|name| name.to_str())
                        .and_then(|name| Some(name.to_string()));
                    let full_name = f
                        .path()
                        .file_name()
                        .and_then(|full_name| full_name.to_str())
                        .and_then(|full_name| Some(full_name.to_string()));
                    let is_file = match &full_name {
                        Some(full_name) => full_name.ends_with(".surql"),
                        None => false,
                    };

                    match name {
                        Some(name) => Some(MigrationFile { name, is_file }),
                        None => None,
                    }
                })
                .collect::<Vec<_>>()
        }
        None => {
            let folder_path = config::retrieve_folder_path();
            let migrations_dir_path = concat_path(&folder_path, MIGRATIONS_DIR_NAME);

            let mut config = HashSet::new();
            config.insert(DirEntryAttr::Name);
            config.insert(DirEntryAttr::IsFile);

            let files = fs_extra::dir::ls(migrations_dir_path, &config)
                .context("Error listing migrations directory")?
                .items;

            let files = files.iter().collect::<Vec<_>>();

            files
                .iter()
                .filter_map(|f| {
                    let is_file = f.get(&DirEntryAttr::IsFile);
                    let is_file = match is_file {
                        Some(is_file) => match is_file {
                            DirEntryValue::Boolean(is_file) => Some(is_file),
                            _ => None,
                        },
                        _ => None,
                    };

                    let name = f.get(&DirEntryAttr::Name);
                    let name = match name {
                        Some(name) => match name {
                            DirEntryValue::String(name) => Some(name),
                            _ => None,
                        },
                        _ => None,
                    };

                    match (name, is_file) {
                        (Some(name), Some(is_file)) => Some(MigrationFile {
                            name: name.to_string(),
                            is_file: is_file.clone(),
                        }),
                        _ => None,
                    }
                })
                .collect::<Vec<_>>()
        }
    };

    let migrations_not_applied = get_sorted_migrations_files(migrations_files)
        .into_iter()
        .filter(|migration_file| {
            is_migration_file_already_applied(migration_file, &migrations_applied).unwrap_or(false)
        })
        .collect::<Vec<_>>();

    let last_migration_applied = migrations_applied.last();

    let migrations_not_applied_before_last_applied =
        if let Some(last_migration_applied) = last_migration_applied {
            migrations_not_applied
                .into_iter()
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
            .map(|migration_file| migration_file.name.to_string())
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

fn get_sorted_migrations_files(migrations_files: Vec<MigrationFile>) -> Vec<MigrationFile> {
    let mut sorted_migrations_files = migrations_files;
    sorted_migrations_files.sort_by(|a, b| a.name.cmp(&b.name));

    sorted_migrations_files
}

fn is_migration_file_already_applied(
    migration_file: &MigrationFile,
    migrations_applied: &Vec<ScriptMigration>,
) -> Result<bool> {
    if !migration_file.is_file {
        return Ok(false);
    }

    let has_already_been_applied = migrations_applied
        .iter()
        .any(|migration_applied| migration_applied.script_name == migration_file.name);

    if has_already_been_applied {
        return Ok(false);
    }

    return Ok(true);
}

fn is_migration_file_before_last_applied(
    migration_file: &MigrationFile,
    last_migration_applied: &ScriptMigration,
) -> Result<bool> {
    if !migration_file.is_file {
        return Ok(false);
    }

    Ok(migration_file.name < last_migration_applied.script_name)
}
