use assert_fs::TempDir;
use color_eyre::eyre::Result;
use pretty_assertions::assert_eq;
use std::fs;

use crate::helpers::*;

#[test]
fn create_migration_file() -> Result<()> {
    let temp_dir = TempDir::new()?;

    scaffold_empty_template(&temp_dir, false)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("create").arg("migration").arg("AddPost");

    cmd.assert().success();

    let migrations_folder = fs::read_dir(temp_dir.join("migrations"))?;
    assert_eq!(migrations_folder.count(), 1);

    temp_dir.close()?;

    Ok(())
}

#[test]
fn create_migration_file_with_down_file() -> Result<()> {
    let temp_dir = TempDir::new()?;

    scaffold_empty_template(&temp_dir, false)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("create")
        .arg("migration")
        .arg("AddPost")
        .arg("--down");

    cmd.assert().success();

    let migrations_dir = temp_dir.join("migrations");

    let migration_files = fs::read_dir(&migrations_dir)?.filter(|entry| match entry.as_ref() {
        Ok(entry) => entry.path().is_file(),
        Err(_) => false,
    });
    assert_eq!(migration_files.count(), 1);

    let migrations_down_dir = migrations_dir.join("down");

    let down_migration_files =
        fs::read_dir(migrations_down_dir)?.filter(|entry| match entry.as_ref() {
            Ok(entry) => entry.path().is_file(),
            Err(_) => false,
        });
    assert_eq!(down_migration_files.count(), 1);

    temp_dir.close()?;

    Ok(())
}

#[test]
fn create_migration_file_with_one_line_content() -> Result<()> {
    let temp_dir = TempDir::new()?;

    scaffold_empty_template(&temp_dir, false)?;

    let mut cmd = create_cmd(&temp_dir)?;

    let content = "CREATE post SET title = 'Hello world!', content = 'This is my first post!', author = user:admin;";

    cmd.arg("create")
        .arg("migration")
        .arg("AddPost")
        .arg("--content")
        .arg(content);

    cmd.assert().success();

    let migrations_folder = fs::read_dir(temp_dir.join("migrations"))?;
    assert_eq!(migrations_folder.count(), 1);

    let migrations_folder = fs::read_dir(temp_dir.join("migrations"))?;
    let migration_file = migrations_folder.into_iter().next().unwrap()?;

    let migration_file_content = fs::read_to_string(migration_file.path())?;

    assert_eq!(migration_file_content, content);

    temp_dir.close()?;

    Ok(())
}

#[test]
fn create_migration_file_with_multiline_content() -> Result<()> {
    let temp_dir = TempDir::new()?;

    scaffold_empty_template(&temp_dir, false)?;

    let mut cmd = create_cmd(&temp_dir)?;

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

    let migrations_folder = fs::read_dir(temp_dir.join("migrations"))?;
    assert_eq!(migrations_folder.count(), 1);

    let migrations_folder = fs::read_dir(temp_dir.join("migrations"))?;
    let migration_file = migrations_folder.into_iter().next().unwrap()?;

    let migration_file_content = fs::read_to_string(migration_file.path())?;

    assert_eq!(migration_file_content, content);

    temp_dir.close()?;

    Ok(())
}
