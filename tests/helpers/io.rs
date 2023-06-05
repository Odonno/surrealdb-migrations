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

pub fn add_post_migration_file() -> Result<()> {
    let content = "CREATE post SET title = 'Hello world!', content = 'This is my first post!', author = user:admin;";

    let mut cmd = create_cmd()?;
    cmd.arg("create")
        .arg("migration")
        .arg("AddPost")
        .arg("--content")
        .arg(content)
        .arg("--down");
    cmd.assert().try_success()?;

    Ok(())
}

pub fn write_post_migration_down_file(migration_name: &str) -> Result<()> {
    let content = "DELETE post;";
    let migration_down_file =
        Path::new("tests-files/migrations/down").join(format!("{}.surql", migration_name));

    fs::write(migration_down_file, content)?;

    Ok(())
}

pub fn add_category_schema_file() -> Result<()> {
    let schemas_files_dir = Path::new("tests-files/schemas");

    if schemas_files_dir.exists() {
        let schema_file = schemas_files_dir.join("category.surql");
        const CONTENT: &str = "DEFINE TABLE category SCHEMALESS;

DEFINE FIELD name ON category TYPE string;
DEFINE FIELD created_at ON category TYPE datetime VALUE $before OR time::now();";

        fs::write(schema_file, CONTENT)?;
    }

    Ok(())
}

pub fn add_category_migration_file() -> Result<()> {
    let content = "CREATE category SET name = 'Technology';
CREATE category SET name = 'Marketing';
CREATE category SET name = 'Books';";

    let mut cmd = create_cmd()?;
    cmd.arg("create")
        .arg("migration")
        .arg("AddCategories")
        .arg("--content")
        .arg(content)
        .arg("--down");
    cmd.assert().try_success()?;

    Ok(())
}

pub fn write_category_migration_down_file(migration_name: &str) -> Result<()> {
    let content = "DELETE category;";
    let migration_down_file =
        Path::new("tests-files/migrations/down").join(format!("{}.surql", migration_name));

    fs::write(migration_down_file, content)?;

    Ok(())
}

pub fn add_archive_schema_file() -> Result<()> {
    let schemas_files_dir = Path::new("tests-files/schemas");

    if schemas_files_dir.exists() {
        let schema_file = schemas_files_dir.join("archive.surql");
        const CONTENT: &str = "DEFINE TABLE archive SCHEMALESS;

DEFINE FIELD name ON archive TYPE string;
DEFINE FIELD from_date ON archive TYPE datetime;
DEFINE FIELD to_date ON archive TYPE datetime;
DEFINE FIELD created_at ON archive TYPE datetime VALUE $before OR time::now();";

        fs::write(schema_file, CONTENT)?;
    }

    Ok(())
}

pub fn add_archive_migration_file() -> Result<()> {
    let content =
        "CREATE archive SET name = '2022', from_date = '2022-01-01', to_date = '2022-12-31';";

    let mut cmd = create_cmd()?;
    cmd.arg("create")
        .arg("migration")
        .arg("AddArchive")
        .arg("--content")
        .arg(content)
        .arg("--down");
    cmd.assert().try_success()?;

    Ok(())
}

pub fn write_archive_migration_down_file(migration_name: &str) -> Result<()> {
    let content = "DELETE archive;";
    let migration_down_file =
        Path::new("tests-files/migrations/down").join(format!("{}.surql", migration_name));

    fs::write(migration_down_file, content)?;

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
