use ::surrealdb::{engine::any::Any, Surreal};
use anyhow::{anyhow, Result};
use include_dir::Dir;
use std::path::Path;

use crate::{
    constants::MIGRATIONS_DIR_NAME,
    io::{self, SurqlFile},
    models::ScriptMigration,
    surrealdb,
};

pub struct ValidateVersionArgs<'a> {
    pub db: &'a Surreal<Any>,
    pub dir: Option<&'a Dir<'a>>,
}

pub async fn main(args: ValidateVersionArgs<'_>) -> Result<()> {
    let ValidateVersionArgs { db: client, dir } = args;

    let migrations_applied =
        surrealdb::list_script_migration_ordered_by_execution_date(client).await?;

    let migrations_dir = Path::new(MIGRATIONS_DIR_NAME).to_path_buf();
    let migrations_files = match io::extract_surql_files(migrations_dir, dir).ok() {
        Some(files) => files,
        None => vec![],
    };

    // TODO : Filter .down.surql files

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
                    is_migration_file_before_last_applied(migration_file, last_migration_applied)
                        .unwrap_or(false)
                })
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };

    if !migrations_not_applied_before_last_applied.is_empty() {
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

fn get_sorted_migrations_files(migrations_files: Vec<SurqlFile>) -> Vec<SurqlFile> {
    let mut sorted_migrations_files = migrations_files;
    sorted_migrations_files.sort_by(|a, b| a.name.cmp(&b.name));

    sorted_migrations_files
}

fn is_migration_file_already_applied(
    migration_file: &SurqlFile,
    migrations_applied: &[ScriptMigration],
) -> Result<bool> {
    let has_already_been_applied = migrations_applied
        .iter()
        .any(|migration_applied| migration_applied.script_name == migration_file.name);

    if has_already_been_applied {
        return Ok(false);
    }

    Ok(true)
}

fn is_migration_file_before_last_applied(
    migration_file: &SurqlFile,
    last_migration_applied: &ScriptMigration,
) -> Result<bool> {
    Ok(migration_file.name < last_migration_applied.script_name)
}
