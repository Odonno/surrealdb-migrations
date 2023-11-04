use assert_fs::TempDir;
use color_eyre::eyre::Result;
use predicates::prelude::*;
use std::path::Path;

use crate::helpers::*;

#[test]
fn scaffold_empty_template() -> Result<()> {
    let temp_dir = TempDir::new()?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("scaffold").arg("template").arg("empty");

    cmd.assert().success();

    assert!(are_folders_equivalent(
        Path::new("templates/empty/schemas"),
        &temp_dir.join("schemas")
    )?);

    let events_dir = temp_dir.join("events");
    assert!(is_empty_folder(&events_dir)?);

    let migrations_dir = temp_dir.join("migrations");
    assert!(is_empty_folder(&migrations_dir)?);

    Ok(())
}

#[test]
fn scaffold_blog_template() -> Result<()> {
    let temp_dir = TempDir::new()?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("scaffold").arg("template").arg("blog");

    cmd.assert().success();

    assert!(are_folders_equivalent(
        Path::new("templates/blog/schemas"),
        &temp_dir.join("schemas")
    )?);

    assert!(are_folders_equivalent(
        Path::new("templates/blog/events"),
        &temp_dir.join("events")
    )?);

    let migrations_dir = temp_dir.join("migrations");

    let migration_files =
        std::fs::read_dir(&migrations_dir)?.filter(|entry| match entry.as_ref() {
            Ok(entry) => entry.path().is_file(),
            Err(_) => false,
        });
    assert_eq!(migration_files.count(), 3);

    let down_migrations_dir = migrations_dir.join("down");

    let down_migration_files =
        std::fs::read_dir(down_migrations_dir)?.filter(|entry| match entry.as_ref() {
            Ok(entry) => entry.path().is_file(),
            Err(_) => false,
        });
    assert_eq!(down_migration_files.count(), 3);

    Ok(())
}

#[test]
fn scaffold_ecommerce_template() -> Result<()> {
    let temp_dir = TempDir::new()?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("scaffold").arg("template").arg("ecommerce");

    cmd.assert().success();

    assert!(are_folders_equivalent(
        Path::new("templates/ecommerce/schemas"),
        &temp_dir.join("schemas")
    )?);

    assert!(are_folders_equivalent(
        Path::new("templates/ecommerce/events"),
        &temp_dir.join("events")
    )?);

    let migrations_dir = temp_dir.join("migrations");

    let migration_files =
        std::fs::read_dir(&migrations_dir)?.filter(|entry| match entry.as_ref() {
            Ok(entry) => entry.path().is_file(),
            Err(_) => false,
        });
    assert_eq!(migration_files.count(), 3);

    let down_migrations_dir = migrations_dir.join("down");

    let down_migration_files =
        std::fs::read_dir(down_migrations_dir)?.filter(|entry| match entry.as_ref() {
            Ok(entry) => entry.path().is_file(),
            Err(_) => false,
        });
    assert_eq!(down_migration_files.count(), 3);

    Ok(())
}

#[test]
fn scaffold_fails_if_schemas_folder_already_exists() -> Result<()> {
    let temp_dir = TempDir::new()?;

    fs_extra::dir::create(temp_dir.join("schemas"), false)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("scaffold").arg("template").arg("blog");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("'schemas' folder already exists."));

    Ok(())
}

#[test]
fn scaffold_fails_if_events_folder_already_exists() -> Result<()> {
    let temp_dir = TempDir::new()?;

    fs_extra::dir::create(temp_dir.join("events"), false)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("scaffold").arg("template").arg("blog");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("'events' folder already exists."));

    Ok(())
}

#[test]
fn scaffold_fails_if_migrations_folder_already_exists() -> Result<()> {
    let temp_dir = TempDir::new()?;

    fs_extra::dir::create(temp_dir.join("migrations"), false)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("scaffold").arg("template").arg("blog");

    cmd.assert().failure().stderr(predicate::str::contains(
        "'migrations' folder already exists.",
    ));

    Ok(())
}

#[test]
fn scaffold_fails_if_invalid_template_name() -> Result<()> {
    let temp_dir = TempDir::new()?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("scaffold").arg("template").arg("invalid");

    cmd.assert().failure();

    Ok(())
}
