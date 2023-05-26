use anyhow::{ensure, Context, Result};
use fs_extra::dir::{DirEntryAttr, DirEntryValue};
use serial_test::serial;
use std::{collections::HashSet, path::Path};

use crate::helpers::*;

#[test]
#[serial]
fn initial_definition_on_initial_schema_changes() -> Result<()> {
    run_with_surreal_instance(|| {
        clear_tests_files()?;
        scaffold_blog_template()?;
        remove_folder("tests-files/migrations")?;

        let mut cmd = create_cmd()?;

        cmd.arg("apply");

        cmd.assert().try_success()?;

        let definitions_files =
            std::fs::read_dir("tests-files/migrations/definitions")?.filter(|entry| {
                match entry.as_ref() {
                    Ok(entry) => entry.path().is_file(),
                    Err(_) => false,
                }
            });
        ensure!(definitions_files.count() == 1);

        ensure!(Path::new("tests-files/migrations/definitions/_initial.json").exists());

        Ok(())
    })
}

#[test]
#[serial]
fn initial_definition_on_initial_migrations() -> Result<()> {
    run_with_surreal_instance(|| {
        clear_tests_files()?;
        scaffold_blog_template()?;

        let mut cmd = create_cmd()?;

        cmd.arg("apply");

        cmd.assert().try_success()?;

        let definitions_files =
            std::fs::read_dir("tests-files/migrations/definitions")?.filter(|entry| {
                match entry.as_ref() {
                    Ok(entry) => entry.path().is_file(),
                    Err(_) => false,
                }
            });
        ensure!(definitions_files.count() == 1);

        ensure!(Path::new("tests-files/migrations/definitions/_initial.json").exists());

        Ok(())
    })
}

#[test]
#[serial]
fn create_new_definition_on_new_migrations() -> Result<()> {
    run_with_surreal_instance(|| {
        clear_tests_files()?;
        scaffold_blog_template()?;
        apply_migrations()?;
        add_new_schema_file()?;
        add_new_migration_file()?;

        let mut cmd = create_cmd()?;

        cmd.arg("apply");

        cmd.assert().try_success()?;

        let mut config = HashSet::new();
        config.insert(DirEntryAttr::Name);
        config.insert(DirEntryAttr::Path);
        config.insert(DirEntryAttr::FullName);

        let mut definitions_files =
            fs_extra::dir::ls("tests-files/migrations/definitions", &config)
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

        ensure!(definitions_files.len() == 2);

        let initial_definition_file = definitions_files
            .first()
            .context("No initial definition file found")?;

        let initial_definition_full_name =
            match initial_definition_file.get(&DirEntryAttr::FullName) {
                Some(DirEntryValue::String(value)) => Some(value),
                _ => None,
            };

        ensure!(initial_definition_full_name == Some(&"_initial.json".to_string()));

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

        ensure!(!new_definition_file_content.is_empty());

        Ok(())
    })
}
