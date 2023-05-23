use ::surrealdb::{engine::any::Any, Surreal};
use anyhow::{anyhow, Result};
use include_dir::Dir;

use crate::{
    io::{self, SurqlFile},
    models::ScriptMigration,
    surrealdb,
};

pub struct ValidateVersionOrderArgs<'a> {
    pub db: &'a Surreal<Any>,
    pub dir: Option<&'a Dir<'static>>,
}

pub async fn main(args: ValidateVersionOrderArgs<'_>) -> Result<()> {
    let ValidateVersionOrderArgs { db: client, dir } = args;

    let migrations_applied =
        surrealdb::list_script_migration_ordered_by_execution_date(client).await?;

    let forward_migrations_files = io::extract_forward_migrations_files(dir);

    let migrations_not_applied = forward_migrations_files
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
