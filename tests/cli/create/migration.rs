use anyhow::Result;
use pretty_assertions::assert_eq;
use serial_test::serial;

use crate::helpers::*;

#[test]
#[serial]
fn create_migration_file() -> Result<()> {
    clear_tests_files()?;
    scaffold_empty_template()?;

    let mut cmd = create_cmd()?;

    cmd.arg("create").arg("migration").arg("AddPost");

    cmd.assert().success();

    let migrations_folder = std::fs::read_dir("tests-files/migrations")?;
    assert_eq!(migrations_folder.count(), 1);

    Ok(())
}

#[test]
#[serial]
fn create_migration_file_with_down_file() -> Result<()> {
    clear_tests_files()?;
    scaffold_empty_template()?;

    let mut cmd = create_cmd()?;

    cmd.arg("create")
        .arg("migration")
        .arg("AddPost")
        .arg("--down");

    cmd.assert().success();

    let migrations_folder = std::fs::read_dir("tests-files/migrations")?;
    let migration_files = migrations_folder.filter(|entry| match entry {
        Ok(entry) => entry.file_type().unwrap().is_file(),
        Err(_) => false,
    });
    assert_eq!(migration_files.count(), 1);

    let down_migrations_folder = std::fs::read_dir("tests-files/migrations/down")?;
    let down_migration_files = down_migrations_folder.filter(|entry| match entry {
        Ok(entry) => entry.file_type().unwrap().is_file(),
        Err(_) => false,
    });
    assert_eq!(down_migration_files.count(), 1);

    Ok(())
}
