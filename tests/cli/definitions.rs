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

        const INITIAL_DEFINITION_FILE_PATH: &str =
            "tests-files/migrations/definitions/_initial.json";

        ensure!(Path::new(INITIAL_DEFINITION_FILE_PATH).exists());

        let initial_migration_definition_str =
            std::fs::read_to_string(INITIAL_DEFINITION_FILE_PATH)?;
        let initial_migration_definition =
            serde_json::from_str::<MigrationDefinition>(&initial_migration_definition_str)?;

        ensure!(
            initial_migration_definition.schemas == Some(INITIAL_DEFINITION_SCHEMAS.to_string())
        );

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
        add_category_schema_file()?;
        add_category_migration_file()?;

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

const INITIAL_DEFINITION_SCHEMAS: &str = "# in: user
# out: post, comment
DEFINE TABLE comment SCHEMALESS
    PERMISSIONS
        FOR select FULL
        FOR create WHERE permission:create_comment IN $auth.permissions
        FOR update, delete WHERE in = $auth.id;

DEFINE FIELD content ON comment TYPE string ASSERT $value != NONE;
DEFINE FIELD created_at ON comment TYPE datetime VALUE $before OR time::now();
DEFINE TABLE permission SCHEMAFULL
    PERMISSIONS
        FOR select FULL
        FOR create, update, delete NONE;

DEFINE FIELD name ON permission TYPE string;
DEFINE FIELD created_at ON permission TYPE datetime VALUE $before OR time::now();

DEFINE INDEX unique_name ON permission COLUMNS name UNIQUE;
DEFINE TABLE post SCHEMALESS
    PERMISSIONS
        FOR select FULL
        FOR create WHERE permission:create_post IN $auth.permissions
        FOR update, delete WHERE author = $auth.id;

DEFINE FIELD title ON post TYPE string;
DEFINE FIELD content ON post TYPE string;
DEFINE FIELD author ON post TYPE record (user) ASSERT $value != NONE;
DEFINE FIELD created_at ON post TYPE datetime VALUE $before OR time::now();
DEFINE FIELD status ON post TYPE string VALUE $value OR $before OR 'DRAFT' ASSERT $value == NONE OR $value INSIDE ['DRAFT', 'PUBLISHED'];
DEFINE TABLE script_migration SCHEMAFULL
    PERMISSIONS
        FOR select FULL
        FOR create, update, delete NONE;

DEFINE FIELD script_name ON script_migration TYPE string;
DEFINE FIELD executed_at ON script_migration TYPE datetime VALUE $before OR time::now();
DEFINE TABLE user SCHEMAFULL
    PERMISSIONS
        FOR select FULL
        FOR update WHERE id = $auth.id
        FOR create, delete NONE;

DEFINE FIELD username ON user TYPE string ASSERT $value != NONE;
DEFINE FIELD email ON user TYPE string ASSERT is::email($value);
DEFINE FIELD password ON user TYPE string ASSERT $value != NONE;
DEFINE FIELD registered_at ON user TYPE datetime VALUE $before OR time::now();
DEFINE FIELD avatar ON user TYPE string;

DEFINE FIELD permissions ON user TYPE array VALUE [permission:create_post, permission:create_comment];
DEFINE FIELD permissions.* ON user TYPE record (permission);

DEFINE INDEX unique_username ON user COLUMNS username UNIQUE;
DEFINE INDEX unique_email ON user COLUMNS email UNIQUE;

DEFINE SCOPE user_scope
    SESSION 30d
    SIGNUP (
        CREATE user
        SET
            username = $username,
            email = $email,
            avatar = \"https://www.gravatar.com/avatar/\" + crypto::md5($email),
            password = crypto::argon2::generate($password)
    )
    SIGNIN (
        SELECT *
        FROM user
        WHERE username = $username AND crypto::argon2::compare(password, $password)
    );";
