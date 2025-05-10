use ::surrealdb::{Connection, Surreal};
use color_eyre::eyre::{eyre, Result};
use include_dir::Dir;
use std::{collections::HashSet, path::Path};

use crate::{
    apply::get_transaction_action,
    common::get_migration_display_name,
    constants::ALL_TAGS,
    io,
    models::MigrationDirection,
    surrealdb::{self},
    validate_checksum::{self, ValidateChecksumArgs},
    validate_version_order::{self, ValidateVersionOrderArgs},
};

pub struct RedoArgs<'a, C: Connection> {
    pub migration_script: String,
    pub db: &'a Surreal<C>,
    pub dir: Option<&'a Dir<'static>>,
    pub display_logs: bool,
    pub dry_run: bool,
    pub validate_checksum: bool,
    pub validate_version_order: bool,
    pub config_file: Option<&'a Path>,
    pub output: bool,
}

pub async fn main<C: Connection>(args: RedoArgs<'_, C>) -> Result<()> {
    let RedoArgs {
        migration_script,
        db: client,
        dir,
        display_logs,
        dry_run,
        validate_checksum,
        validate_version_order,
        config_file,
        output,
    } = args;

    if validate_version_order {
        let validate_version_order_args = ValidateVersionOrderArgs {
            db: client,
            dir,
            config_file,
        };
        validate_version_order::main(validate_version_order_args).await?;
    }

    if validate_checksum {
        let validate_checksum_args = ValidateChecksumArgs {
            db: client,
            dir,
            config_file,
        };
        validate_checksum::main(validate_checksum_args).await?;
    }

    let display_logs = match dry_run {
        true => false,
        false => display_logs,
    };

    let tags = HashSet::from([ALL_TAGS.into()]);

    let forward_migrations_files =
        io::extract_migrations_files(config_file, dir, MigrationDirection::Forward, &tags);

    let migration_file = forward_migrations_files
        .into_iter()
        .find(|f| f.name == migration_script || f.full_name == migration_script);

    let Some(migration_file) = migration_file else {
        return Err(eyre!("The migration file was not found"));
    };

    let migrations_applied =
        surrealdb::list_script_migration_ordered_by_execution_date(client).await?;

    let is_migration_already_applied = migrations_applied
        .iter()
        .any(|m| m.script_name == migration_file.name || m.script_name == migration_file.full_name);

    if !is_migration_already_applied {
        return Err(eyre!("This migration was not applied in the SurrealDB instance. Please make sure you correctly applied this migration before."));
    }

    let migration_content = migration_file.get_content().unwrap_or(String::new());
    let statements = surrealdb::parse_statements(&migration_content)?;

    if output {
        println!("{}", statements);
    }

    if display_logs {
        let migration_display_name = get_migration_display_name(&migration_file.name);
        println!("Executing migration {}...", migration_display_name);
    }

    let transaction_action = get_transaction_action(dry_run);
    surrealdb::apply_in_transaction(client, statements.0 .0, transaction_action).await?;

    if display_logs {
        println!("Migration successfully re-executed!");
    }

    Ok(())
}
