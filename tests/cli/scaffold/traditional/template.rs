use assert_fs::TempDir;
use color_eyre::eyre::Result;
use insta::{assert_snapshot, Settings};

use crate::helpers::*;

#[test]
fn scaffold_empty_template() -> Result<()> {
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
fn scaffold_blog_template() -> Result<()> {
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
fn scaffold_ecommerce_template() -> Result<()> {
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
