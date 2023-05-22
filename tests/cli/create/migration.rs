use anyhow::Result;
use pretty_assertions::assert_eq;
use serial_test::serial;
use std::fs;

use crate::helpers::*;

#[test]
#[serial]
fn create_migration_file() -> Result<()> {
    clear_tests_files()?;
    scaffold_empty_template()?;

    let mut cmd = create_cmd()?;

    cmd.arg("create").arg("migration").arg("AddPost");

    cmd.assert().success();

    let migrations_folder = fs::read_dir("tests-files/migrations")?;
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

    let migration_files =
        fs::read_dir("tests-files/migrations")?.filter(|entry| match entry.as_ref() {
            Ok(entry) => entry.path().is_file(),
            Err(_) => false,
        });
    assert_eq!(migration_files.count(), 1);

    let down_migration_files =
        fs::read_dir("tests-files/migrations/down")?.filter(|entry| match entry.as_ref() {
            Ok(entry) => entry.path().is_file(),
            Err(_) => false,
        });
    assert_eq!(down_migration_files.count(), 1);

    Ok(())
}

#[test]
#[serial]
fn create_migration_file_with_one_line_content() -> Result<()> {
    clear_tests_files()?;
    scaffold_empty_template()?;

    let mut cmd = create_cmd()?;

    let content = "CREATE post SET title = 'Hello world!', content = 'This is my first post!', author = user:admin;";

    cmd.arg("create")
        .arg("migration")
        .arg("AddPost")
        .arg("--content")
        .arg(content);

    cmd.assert().success();

    let migrations_folder = fs::read_dir("tests-files/migrations")?;
    assert_eq!(migrations_folder.count(), 1);

    let migrations_folder = fs::read_dir("tests-files/migrations")?;
    let migration_file = migrations_folder.into_iter().next().unwrap()?;

    let migration_file_content = fs::read_to_string(migration_file.path())?;

    assert_eq!(migration_file_content, content);

    Ok(())
}

#[test]
#[serial]
fn create_migration_file_with_multiline_content() -> Result<()> {
    clear_tests_files()?;
    scaffold_empty_template()?;

    let mut cmd = create_cmd()?;

    let content = "CREATE post SET title = 'Hello world!', content = 'This is my first post!', author = user:admin;
CREATE post SET title = 'Hello world!', content = 'This is my first post!', author = user:admin;
CREATE post SET title = 'Hello world!', content = 'This is my first post!', author = user:admin;
CREATE post SET title = 'Hello world!', content = 'This is my first post!', author = user:admin;";

    cmd.arg("create")
        .arg("migration")
        .arg("AddPost")
        .arg("--content")
        .arg(content);

    cmd.assert().success();

    let migrations_folder = fs::read_dir("tests-files/migrations")?;
    assert_eq!(migrations_folder.count(), 1);

    let migrations_folder = fs::read_dir("tests-files/migrations")?;
    let migration_file = migrations_folder.into_iter().next().unwrap()?;

    let migration_file_content = fs::read_to_string(migration_file.path())?;

    assert_eq!(migration_file_content, content);

    Ok(())
}
