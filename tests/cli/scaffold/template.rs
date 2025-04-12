use assert_fs::TempDir;
use color_eyre::eyre::Result;
use insta::{assert_snapshot, Settings};
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

    temp_dir.close()?;

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

    temp_dir.close()?;

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

    temp_dir.close()?;

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

    temp_dir.close()?;

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

    temp_dir.close()?;

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

    temp_dir.close()?;

    Ok(())
}

#[test]
fn scaffold_fails_if_invalid_template_name() -> Result<()> {
    let temp_dir = TempDir::new()?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("scaffold").arg("template").arg("invalid");

    cmd.assert().failure();

    temp_dir.close()?;

    Ok(())
}

#[test]
fn scaffold_empty_template_with_traditional_approach() -> Result<()> {
    let temp_dir = TempDir::new()?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("scaffold")
        .arg("template")
        .arg("empty")
        .arg("--traditional");

    cmd.assert().success();

    let schemas_dir = temp_dir.join("schemas");
    assert!(!is_folder_exists(&schemas_dir)?);

    let events_dir = temp_dir.join("events");
    assert!(!is_folder_exists(&events_dir)?);

    let migrations_dir = temp_dir.join("migrations");

    {
        let migration_files =
            std::fs::read_dir(&migrations_dir)?.filter(|entry| match entry.as_ref() {
                Ok(entry) => entry.path().is_file(),
                Err(_) => false,
            });
        assert_eq!(migration_files.count(), 1);

        let initial_file = std::fs::read_dir(&migrations_dir)?.find(|entry| match entry.as_ref() {
            Ok(entry) => entry.file_name() == "__Initial.surql",
            Err(_) => false,
        });

        assert!(initial_file.is_some());

        let initial_file = initial_file.unwrap().unwrap();

        assert_eq!(initial_file.file_name(), "__Initial.surql");

        let initial_content = std::fs::read_to_string(initial_file.path())?;

        assert_snapshot!(initial_content);
    }

    {
        let down_migrations_dir = migrations_dir.join("down");

        let down_migration_files =
            std::fs::read_dir(&down_migrations_dir)?.filter(|entry| match entry.as_ref() {
                Ok(entry) => entry.path().is_file(),
                Err(_) => false,
            });
        assert_eq!(down_migration_files.count(), 1);

        let initial_down_file =
            std::fs::read_dir(&down_migrations_dir)?.find(|entry| match entry.as_ref() {
                Ok(entry) => entry.file_name() == "__Initial.surql",
                Err(_) => false,
            });

        assert!(initial_down_file.is_some());

        let initial_down_file = initial_down_file.unwrap().unwrap();

        assert_eq!(initial_down_file.file_name(), "__Initial.surql");

        let initial_down_content = std::fs::read_to_string(initial_down_file.path())?;

        assert_snapshot!(initial_down_content);
    }

    temp_dir.close()?;

    Ok(())
}

#[test]
fn scaffold_blog_template_with_traditional_approach() -> Result<()> {
    let temp_dir = TempDir::new()?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("scaffold")
        .arg("template")
        .arg("blog")
        .arg("--traditional");

    cmd.assert().success();

    let schemas_dir = temp_dir.join("schemas");
    assert!(!is_folder_exists(&schemas_dir)?);

    let events_dir = temp_dir.join("events");
    assert!(!is_folder_exists(&events_dir)?);

    let migrations_dir = temp_dir.join("migrations");

    {
        let migration_files =
            std::fs::read_dir(&migrations_dir)?.filter(|entry| match entry.as_ref() {
                Ok(entry) => entry.path().is_file(),
                Err(_) => false,
            });
        assert_eq!(migration_files.count(), 4);

        let initial_file = std::fs::read_dir(&migrations_dir)?.find(|entry| match entry.as_ref() {
            Ok(entry) => entry.file_name() == "__Initial.surql",
            Err(_) => false,
        });

        assert!(initial_file.is_some());

        let initial_file = initial_file.unwrap().unwrap();

        assert_eq!(initial_file.file_name(), "__Initial.surql");

        let initial_content = std::fs::read_to_string(initial_file.path())?;

        let mut settings = Settings::new();
        settings.add_filter(r"'[0-9a-zA-Z]{128}'", "[jwt_key]");

        settings.bind(|| {
            assert_snapshot!(initial_content);
        });
    }

    {
        let down_migrations_dir = migrations_dir.join("down");

        let down_migration_files =
            std::fs::read_dir(&down_migrations_dir)?.filter(|entry| match entry.as_ref() {
                Ok(entry) => entry.path().is_file(),
                Err(_) => false,
            });
        assert_eq!(down_migration_files.count(), 4);

        let initial_down_file =
            std::fs::read_dir(&down_migrations_dir)?.find(|entry| match entry.as_ref() {
                Ok(entry) => entry.file_name() == "__Initial.surql",
                Err(_) => false,
            });

        assert!(initial_down_file.is_some());

        let initial_down_file = initial_down_file.unwrap().unwrap();

        assert_eq!(initial_down_file.file_name(), "__Initial.surql");

        let initial_down_content = std::fs::read_to_string(initial_down_file.path())?;

        assert_snapshot!(initial_down_content);
    }

    temp_dir.close()?;

    Ok(())
}

#[test]
fn scaffold_ecommerce_template_with_traditional_approach() -> Result<()> {
    let temp_dir = TempDir::new()?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("scaffold")
        .arg("template")
        .arg("ecommerce")
        .arg("--traditional");

    cmd.assert().success();

    let schemas_dir = temp_dir.join("schemas");
    assert!(!is_folder_exists(&schemas_dir)?);

    let events_dir = temp_dir.join("events");
    assert!(!is_folder_exists(&events_dir)?);

    let migrations_dir = temp_dir.join("migrations");

    {
        let migration_files =
            std::fs::read_dir(&migrations_dir)?.filter(|entry| match entry.as_ref() {
                Ok(entry) => entry.path().is_file(),
                Err(_) => false,
            });
        assert_eq!(migration_files.count(), 4);

        let initial_file = std::fs::read_dir(&migrations_dir)?.find(|entry| match entry.as_ref() {
            Ok(entry) => entry.file_name() == "__Initial.surql",
            Err(_) => false,
        });

        assert!(initial_file.is_some());

        let initial_file = initial_file.unwrap().unwrap();

        assert_eq!(initial_file.file_name(), "__Initial.surql");

        let initial_content = std::fs::read_to_string(initial_file.path())?;

        let mut settings = Settings::new();
        settings.add_filter(r"'[0-9a-zA-Z]{128}'", "[jwt_key]");

        settings.bind(|| {
            assert_snapshot!(initial_content);
        });
    }

    {
        let down_migrations_dir = migrations_dir.join("down");

        let down_migration_files =
            std::fs::read_dir(&down_migrations_dir)?.filter(|entry| match entry.as_ref() {
                Ok(entry) => entry.path().is_file(),
                Err(_) => false,
            });
        assert_eq!(down_migration_files.count(), 4);

        let initial_down_file =
            std::fs::read_dir(&down_migrations_dir)?.find(|entry| match entry.as_ref() {
                Ok(entry) => entry.file_name() == "__Initial.surql",
                Err(_) => false,
            });

        assert!(initial_down_file.is_some());

        let initial_down_file = initial_down_file.unwrap().unwrap();

        assert_eq!(initial_down_file.file_name(), "__Initial.surql");

        let initial_down_content = std::fs::read_to_string(initial_down_file.path())?;

        assert_snapshot!(initial_down_content);
    }

    temp_dir.close()?;

    Ok(())
}
