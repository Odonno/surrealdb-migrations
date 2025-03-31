use assert_fs::TempDir;
use color_eyre::{
    eyre::{ensure, ContextCompat, WrapErr},
    Result,
};
use fs_extra::dir::{DirEntryAttr, DirEntryValue};
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

    ensure!(
        initial_migration_definition.schemas == Some(INITIAL_DEFINITION_SCHEMAS.to_string()),
        "Expected initial definition to be equal to the initial definition schema"
    );

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

const INITIAL_DEFINITION_SCHEMAS: &str = "# in: user
# out: post, comment
DEFINE TABLE OVERWRITE comment SCHEMALESS
    PERMISSIONS
        FOR select FULL
        FOR create WHERE permission:create_comment IN $auth.permissions
        FOR update, delete WHERE in = $auth.id;

DEFINE FIELD OVERWRITE content ON comment TYPE string;
DEFINE FIELD OVERWRITE created_at ON comment TYPE datetime VALUE time::now() READONLY;
DEFINE TABLE OVERWRITE permission SCHEMAFULL
    PERMISSIONS
        FOR select FULL
        FOR create, update, delete NONE;

DEFINE FIELD OVERWRITE name ON permission TYPE string;
DEFINE FIELD OVERWRITE created_at ON permission TYPE datetime VALUE time::now() READONLY;

DEFINE INDEX OVERWRITE unique_name ON permission COLUMNS name UNIQUE;
DEFINE TABLE OVERWRITE post SCHEMALESS
    PERMISSIONS
        FOR select FULL
        FOR create WHERE permission:create_post IN $auth.permissions
        FOR update, delete WHERE author = $auth.id;

DEFINE FIELD OVERWRITE title ON post TYPE string;
DEFINE FIELD OVERWRITE content ON post TYPE string;
DEFINE FIELD OVERWRITE author ON post TYPE references<user>;
DEFINE FIELD OVERWRITE created_at ON post TYPE datetime VALUE time::now() READONLY;
DEFINE FIELD OVERWRITE status ON post TYPE string DEFAULT 'DRAFT' ASSERT $value IN ['DRAFT', 'PUBLISHED'];
DEFINE TABLE OVERWRITE script_migration SCHEMAFULL
    PERMISSIONS
        FOR select FULL
        FOR create, update, delete NONE;

DEFINE FIELD OVERWRITE script_name ON script_migration TYPE string;
DEFINE FIELD OVERWRITE executed_at ON script_migration TYPE datetime VALUE time::now() READONLY;
DEFINE TABLE OVERWRITE user SCHEMAFULL
    PERMISSIONS
        FOR select FULL
        FOR update WHERE id = $auth.id
        FOR create, delete NONE;

DEFINE FIELD OVERWRITE username ON user TYPE string;
DEFINE FIELD OVERWRITE email ON user TYPE string ASSERT string::is::email($value);
DEFINE FIELD OVERWRITE password ON user TYPE string;
DEFINE FIELD OVERWRITE registered_at ON user TYPE datetime VALUE time::now() READONLY;
DEFINE FIELD OVERWRITE avatar ON user TYPE option<string>;

DEFINE FIELD OVERWRITE permissions ON user TYPE array<record<permission>> 
    DEFAULT [permission:create_post, permission:create_comment];

DEFINE INDEX OVERWRITE unique_username ON user COLUMNS username UNIQUE;
DEFINE INDEX OVERWRITE unique_email ON user COLUMNS email UNIQUE;

DEFINE SCOPE OVERWRITE user_scope
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
