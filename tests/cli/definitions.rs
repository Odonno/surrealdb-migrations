use assert_fs::TempDir;
use color_eyre::{
    eyre::{ensure, ContextCompat, Error, WrapErr},
    Result,
};
use fs_extra::dir::{DirEntryAttr, DirEntryValue};
use insta::{assert_snapshot, Settings};
use std::collections::HashSet;

use crate::helpers::*;

#[test]
fn initial_definition_on_initial_schema_changes() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    let migrations_dir = temp_dir.join("migrations");

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir)?;
    remove_folder(&migrations_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply");

    cmd.assert().try_success()?;

    let definitions_dir = migrations_dir.join("definitions");

    let definitions_files =
        std::fs::read_dir(&definitions_dir)?.filter(|entry| match entry.as_ref() {
            Ok(entry) => entry.path().is_file(),
            Err(_) => false,
        });
    ensure!(
        definitions_files.count() == 1,
        "Expected only one definition file"
    );

    let initial_definition_file_path = definitions_dir.join("_initial.json");

    ensure!(
        initial_definition_file_path.exists(),
        "Expected _initial.json file to exist"
    );

    let initial_migration_definition_str = std::fs::read_to_string(initial_definition_file_path)?;
    let initial_migration_definition =
        serde_json::from_str::<MigrationDefinition>(&initial_migration_definition_str)?;

    let insta_settings = Settings::new();
    insta_settings.bind(|| {
        assert_snapshot!(
            "Initial migration definition",
            initial_migration_definition.schemas.unwrap_or_default()
        );
        Ok::<(), Error>(())
    })?;

    Ok(())
}

#[test]
fn initial_definition_on_initial_migrations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    let migrations_dir = temp_dir.join("migrations");

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply");

    cmd.assert().try_success()?;

    let definitions_dir = migrations_dir.join("definitions");

    let definitions_files =
        std::fs::read_dir(&definitions_dir)?.filter(|entry| match entry.as_ref() {
            Ok(entry) => entry.path().is_file(),
            Err(_) => false,
        });
    ensure!(
        definitions_files.count() == 1,
        "Expected only one definition file"
    );

    let initial_definition_file_path = definitions_dir.join("_initial.json");

    ensure!(
        initial_definition_file_path.exists(),
        "Expected _initial.json file to exist"
    );

    Ok(())
}

#[test]
fn create_new_definition_on_new_migrations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir)?;
    apply_migrations(&temp_dir, &db_name)?;
    add_category_schema_file(&temp_dir)?;
    add_category_migration_file(&temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply");

    cmd.assert().try_success()?;

    let mut config = HashSet::new();
    config.insert(DirEntryAttr::Name);
    config.insert(DirEntryAttr::Path);
    config.insert(DirEntryAttr::FullName);

    let migrations_dir = temp_dir.join("migrations");
    let definitions_dir = migrations_dir.join("definitions");

    let mut definitions_files = fs_extra::dir::ls(definitions_dir, &config)
        .context("Error listing definitions directory")?
        .items;

    definitions_files.sort_by(|a, b| {
        let a = match a.get(&DirEntryAttr::Name) {
            Some(DirEntryValue::String(value)) => Some(value),
            _ => None,
        };

        let b = match b.get(&DirEntryAttr::Name) {
            Some(DirEntryValue::String(value)) => Some(value),
            _ => None,
        };

        b.cmp(&a)
    });

    ensure!(
        definitions_files.len() == 2,
        "Expected two definition files"
    );

    let initial_definition_file = definitions_files
        .first()
        .context("No initial definition file found")?;

    let initial_definition_full_name = match initial_definition_file.get(&DirEntryAttr::FullName) {
        Some(DirEntryValue::String(value)) => Some(value),
        _ => None,
    };

    ensure!(
        initial_definition_full_name == Some(&"_initial.json".to_string()),
        "invalid initial definition file name"
    );

    let new_definition_file = definitions_files
        .last()
        .context("No new definition file found")?;

    let new_definition_path = match new_definition_file.get(&DirEntryAttr::Path) {
        Some(DirEntryValue::String(value)) => Some(value),
        _ => None,
    };

    let new_definition_file_content = match new_definition_path {
        Some(path) => std::fs::read_to_string(path)?,
        _ => "".to_string(),
    };

    ensure!(
        !new_definition_file_content.is_empty(),
        "empty new definition file"
    );

    Ok(())
}
