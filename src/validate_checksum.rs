use ::surrealdb::{Connection, Surreal};
use color_eyre::eyre::{eyre, Result};
use include_dir::Dir;
use sha2::{Digest, Sha256};
use std::path::Path;

use crate::{
    io::{self},
    models::MigrationDirection,
    surrealdb,
};

pub struct ValidateChecksumArgs<'a, C: Connection> {
    pub db: &'a Surreal<C>,
    pub dir: Option<&'a Dir<'static>>,
    pub config_file: Option<&'a Path>,
}

pub async fn main<C: Connection>(args: ValidateChecksumArgs<'_, C>) -> Result<()> {
    let ValidateChecksumArgs {
        db: client,
        dir,
        config_file,
    } = args;

    let migrations_applied =
        surrealdb::list_script_migration_ordered_by_execution_date(client).await?;

    let forward_migrations_files =
        io::extract_migrations_files(config_file, dir, MigrationDirection::Forward);

    for migration_applied in migrations_applied {
        if let Some(checksum) = migration_applied.checksum {
            let migration_file = forward_migrations_files
                .iter()
                .find(|f| f.name == migration_applied.script_name);

            if let Some(migration_file) = migration_file {
                let file_checksum =
                    Sha256::digest(migration_file.get_content().unwrap_or_default()).to_vec();
                let file_checksum = hex::encode(file_checksum);

                if checksum != file_checksum {
                    return Err(eyre!(
                        "The checksum does not match for migration '{}'.",
                        migration_applied.script_name
                    ));
                }
            } else {
                return Err(eyre!(
                    "The migration file '{}' does not exist.",
                    migration_applied.script_name
                ));
            }
        }
    }

    Ok(())
}
