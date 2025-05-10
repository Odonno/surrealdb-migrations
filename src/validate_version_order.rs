use ::surrealdb::{Connection, Surreal};
use color_eyre::eyre::{eyre, Result};
use include_dir::Dir;
use lexicmp::natural_lexical_cmp;
use std::{cmp::Ordering, collections::HashSet, path::Path};

use crate::{
    constants::ALL_TAGS,
    file::SurqlFile,
    io::{self},
    models::{MigrationDirection, ScriptMigration},
    surrealdb,
};

pub struct ValidateVersionOrderArgs<'a, C: Connection> {
    pub db: &'a Surreal<C>,
    pub dir: Option<&'a Dir<'static>>,
    pub config_file: Option<&'a Path>,
}

pub async fn main<C: Connection>(args: ValidateVersionOrderArgs<'_, C>) -> Result<()> {
    let ValidateVersionOrderArgs {
        db: client,
        dir,
        config_file,
    } = args;

    let migrations_applied =
        surrealdb::list_script_migration_ordered_by_execution_date(client).await?;

    let tags = HashSet::from([ALL_TAGS.into()]);

    let forward_migrations_files =
        io::extract_migrations_files(config_file, dir, MigrationDirection::Forward, &tags);

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

        Err(eyre!(
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
    Ok(
        natural_lexical_cmp(&migration_file.name, &last_migration_applied.script_name)
            == Ordering::Less,
    )
}
