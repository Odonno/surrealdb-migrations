use anyhow::{Context, Result};
use std::{fs, path::Path};

use super::create_cmd;

pub fn clear_tests_files() -> Result<()> {
    remove_folder("tests-files")?;
    remove_folder("tests-files-alt")?;

    Ok(())
}

pub fn empty_folder(folder: &str) -> Result<()> {
    let migrations_files_dir = Path::new(folder);

    if migrations_files_dir.exists() {
        fs::remove_dir_all(migrations_files_dir)?;
        fs::create_dir(migrations_files_dir)?;
    }

    Ok(())
}

pub fn remove_folder(folder: &str) -> Result<()> {
    let dir = Path::new(folder);

    if dir.exists() {
        fs::remove_dir_all(dir)?;
    }

    Ok(())
}

pub fn add_new_schema_file() -> Result<()> {
    let schemas_files_dir = Path::new("tests-files/schemas");

    if schemas_files_dir.exists() {
        let category_schema_file = schemas_files_dir.join("category.surql");
        const CATEGORY_CONTENT: &str = "DEFINE TABLE category SCHEMALESS;

DEFINE FIELD name ON category TYPE string;
DEFINE FIELD created_at ON category TYPE datetime VALUE $before OR time::now();";

        fs::write(category_schema_file, CATEGORY_CONTENT)?;
    }

    Ok(())
}

pub fn add_new_migration_file() -> Result<()> {
    let content = "CREATE category SET name = 'Technology';
CREATE category SET name = 'Marketing';
CREATE category SET name = 'Books';";

    let mut cmd = create_cmd()?;
    cmd.arg("create")
        .arg("migration")
        .arg("AddCategories")
        .arg("--content")
        .arg(content);
    cmd.assert().try_success()?;

    Ok(())
}

pub fn inline_down_migration_files() -> Result<()> {
    let migrations_files_dir = Path::new("tests-files/migrations");
    let down_migrations_files_dir = Path::new("tests-files/migrations/down");

    let down_migrations_files = down_migrations_files_dir
        .read_dir()?
        .filter(|entry| match entry.as_ref() {
            Ok(entry) => entry.path().is_file(),
            Err(_) => false,
        })
        .collect::<Vec<_>>();

    for down_migrations_file in down_migrations_files {
        let down_migration_file = down_migrations_file?;
        let down_migration_file_name = down_migration_file.file_name();
        let down_migration_file_name = down_migration_file_name
            .to_str()
            .context("Invalid file name")?;
        let down_migration_file_name = down_migration_file_name.replace(".surql", "");

        let inlined_down_migration_file_name = migrations_files_dir.join(down_migration_file_name);
        let inlined_down_migration_file_name =
            inlined_down_migration_file_name.with_extension("down.surql");

        let down_migration_file_content = fs::read_to_string(down_migration_file.path())?;

        fs::write(
            inlined_down_migration_file_name,
            down_migration_file_content,
        )?;
    }

    remove_folder("tests-files/migrations/down")?;

    Ok(())
}
