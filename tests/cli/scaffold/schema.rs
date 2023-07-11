use anyhow::Result;
use assert_fs::TempDir;
use pretty_assertions::assert_eq;
use std::path::Path;

use crate::helpers::*;

#[test]
fn scaffold_fails_from_empty_schema_file() -> Result<()> {
    let temp_dir = TempDir::new()?;

    copy_folder(Path::new("schema-files"), &temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("scaffold")
        .arg("schema")
        .arg("schema-files/empty.sql")
        .arg("--db-type")
        .arg("mssql");

    cmd.assert()
        .failure()
        .stderr("Error: No table found in schema file.\n");

    Ok(())
}

#[test]
fn scaffold_from_create_table_fails_if_contains_table_named_script_migration() -> Result<()> {
    let temp_dir = TempDir::new()?;

    copy_folder(Path::new("schema-files"), &temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("scaffold")
        .arg("schema")
        .arg("schema-files/mssql/create_table_with_script_migration.sql")
        .arg("--db-type")
        .arg("mssql");

    cmd.assert()
        .failure()
        .stderr("Error: The table 'script_migration' is reserved for internal use.\n");

    Ok(())
}

#[test]
fn scaffold_from_create_table() -> Result<()> {
    let temp_dir = TempDir::new()?;

    copy_folder(Path::new("schema-files"), &temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("scaffold")
        .arg("schema")
        .arg("schema-files/mssql/create_table.sql")
        .arg("--db-type")
        .arg("mssql");

    cmd.assert().success();

    let schemas_dir = temp_dir.join("schemas");

    assert!(schemas_dir.join("script_migration.surql").exists());

    let schema_files = std::fs::read_dir(&schemas_dir)?;
    assert_eq!(schema_files.count(), 2);

    let post_schema = std::fs::read_to_string(schemas_dir.join("post.surql"))?;
    assert_eq!(
        post_schema,
        "DEFINE TABLE post SCHEMALESS;

DEFINE FIELD id ON post ASSERT $value != NONE;
DEFINE FIELD title ON post TYPE string;
DEFINE FIELD content ON post TYPE string;
DEFINE FIELD status ON post TYPE string;
DEFINE FIELD created_at ON post TYPE datetime;
"
    );

    let events_dir = temp_dir.join("events");
    assert!(is_empty_folder(&events_dir)?);

    let migrations_dir = temp_dir.join("migrations");
    assert!(is_empty_folder(&migrations_dir)?);

    Ok(())
}

#[test]
fn scaffold_from_schema_file_but_preserve_casing() -> Result<()> {
    let temp_dir = TempDir::new()?;

    copy_folder(Path::new("schema-files"), &temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("scaffold")
        .arg("schema")
        .arg("schema-files/mssql/create_table.sql")
        .arg("--db-type")
        .arg("mssql")
        .arg("--preserve-casing");

    cmd.assert().success();

    let schemas_dir = temp_dir.join("schemas");

    assert!(schemas_dir.join("script_migration.surql").exists());

    let schema_files = std::fs::read_dir(&schemas_dir)?;
    assert_eq!(schema_files.count(), 2);

    let post_schema = std::fs::read_to_string(schemas_dir.join("Post.surql"))?;
    assert_eq!(
        post_schema,
        "DEFINE TABLE Post SCHEMALESS;

DEFINE FIELD Id ON Post ASSERT $value != NONE;
DEFINE FIELD Title ON Post TYPE string;
DEFINE FIELD Content ON Post TYPE string;
DEFINE FIELD Status ON Post TYPE string;
DEFINE FIELD CreatedAt ON Post TYPE datetime;
"
    );

    let events_dir = temp_dir.join("events");
    assert!(is_empty_folder(&events_dir)?);

    let migrations_dir = temp_dir.join("migrations");
    assert!(is_empty_folder(&migrations_dir)?);

    Ok(())
}

#[test]
fn scaffold_from_create_table_with_many_types() -> Result<()> {
    let temp_dir = TempDir::new()?;

    copy_folder(Path::new("schema-files"), &temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("scaffold")
        .arg("schema")
        .arg("schema-files/mssql/create_table_with_many_types.sql")
        .arg("--db-type")
        .arg("mssql");

    cmd.assert().success();

    let schemas_dir = temp_dir.join("schemas");

    assert!(schemas_dir.join("script_migration.surql").exists());

    let schema_files = std::fs::read_dir(&schemas_dir)?;
    assert_eq!(schema_files.count(), 2);

    let test_schema = std::fs::read_to_string(schemas_dir.join("test.surql"))?;
    assert_eq!(
        test_schema,
        "DEFINE TABLE test SCHEMALESS;

DEFINE FIELD id ON test ASSERT $value != NONE;
DEFINE FIELD char ON test TYPE string;
DEFINE FIELD n_char ON test TYPE string;
DEFINE FIELD varchar ON test TYPE string;
DEFINE FIELD n_varchar ON test TYPE string;
DEFINE FIELD text ON test TYPE string;
DEFINE FIELD bit ON test TYPE bool;
DEFINE FIELD tiny_int ON test TYPE number;
DEFINE FIELD small_int ON test TYPE number;
DEFINE FIELD int ON test TYPE number;
DEFINE FIELD big_int ON test TYPE number;
DEFINE FIELD decimal ON test TYPE number;
DEFINE FIELD numeric ON test TYPE number;
DEFINE FIELD float ON test TYPE number;
DEFINE FIELD date_time ON test TYPE datetime;
DEFINE FIELD timestamp ON test TYPE datetime;
DEFINE FIELD json ON test TYPE object;
DEFINE FIELD variant ON test;
"
    );

    let events_dir = temp_dir.join("events");
    assert!(is_empty_folder(&events_dir)?);

    let migrations_dir = temp_dir.join("migrations");
    assert!(is_empty_folder(&migrations_dir)?);

    Ok(())
}

#[test]
fn scaffold_from_create_multiple_table_with_relations() -> Result<()> {
    let temp_dir = TempDir::new()?;

    copy_folder(Path::new("schema-files"), &temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("scaffold")
        .arg("schema")
        .arg("schema-files/mssql/create_table_with_relations.sql")
        .arg("--db-type")
        .arg("mssql");

    cmd.assert().success();

    let schemas_dir = temp_dir.join("schemas");

    assert!(schemas_dir.join("script_migration.surql").exists());

    let schema_files = std::fs::read_dir(&schemas_dir)?;
    assert_eq!(schema_files.count(), 4);

    let post_schema = std::fs::read_to_string(schemas_dir.join("post.surql"))?;
    assert_eq!(
        post_schema,
        "DEFINE TABLE post SCHEMALESS;

DEFINE FIELD id ON post ASSERT $value != NONE;
DEFINE FIELD title ON post TYPE string ASSERT $value != NONE;
DEFINE FIELD content ON post TYPE string ASSERT $value != NONE;
DEFINE FIELD status ON post TYPE string ASSERT $value != NONE;
DEFINE FIELD created_at ON post TYPE datetime ASSERT $value != NONE;
"
    );

    let user_schema = std::fs::read_to_string(schemas_dir.join("user.surql"))?;
    assert_eq!(
        user_schema,
        "DEFINE TABLE user SCHEMALESS;

DEFINE FIELD id ON user ASSERT $value != NONE;
DEFINE FIELD username ON user TYPE string ASSERT $value != NONE;
DEFINE INDEX user_username_index ON user COLUMNS username UNIQUE;
DEFINE FIELD email ON user TYPE string ASSERT $value != NONE;
DEFINE INDEX user_email_index ON user COLUMNS email UNIQUE;
DEFINE FIELD password ON user TYPE string ASSERT $value != NONE;
DEFINE FIELD registered_at ON user TYPE datetime ASSERT $value != NONE;
"
    );

    let comment_schema = std::fs::read_to_string(schemas_dir.join("comment.surql"))?;
    assert_eq!(
        comment_schema,
        "DEFINE TABLE comment SCHEMALESS;

DEFINE FIELD id ON comment ASSERT $value != NONE;
DEFINE FIELD content ON comment TYPE string ASSERT $value != NONE;
DEFINE FIELD created_at ON comment TYPE datetime ASSERT $value != NONE;
DEFINE FIELD user ON comment TYPE record(user) ASSERT $value != NONE;
DEFINE FIELD post ON comment TYPE record(post) ASSERT $value != NONE;
"
    );

    let events_dir = temp_dir.join("events");
    assert!(is_empty_folder(&events_dir)?);

    let migrations_dir = temp_dir.join("migrations");
    assert!(is_empty_folder(&migrations_dir)?);

    Ok(())
}

#[test]
fn scaffold_from_create_table_with_unique_index() -> Result<()> {
    let temp_dir = TempDir::new()?;

    copy_folder(Path::new("schema-files"), &temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("scaffold")
        .arg("schema")
        .arg("schema-files/mssql/create_table_with_unique_index.sql")
        .arg("--db-type")
        .arg("mssql");

    cmd.assert().success();

    let schemas_dir = temp_dir.join("schemas");

    assert!(schemas_dir.join("script_migration.surql").exists());

    let schema_files = std::fs::read_dir(&schemas_dir)?;
    assert_eq!(schema_files.count(), 2);

    let user_schema = std::fs::read_to_string(schemas_dir.join("user.surql"))?;
    assert_eq!(
        user_schema,
        "DEFINE TABLE user SCHEMALESS;

DEFINE FIELD id ON user ASSERT $value != NONE;
DEFINE FIELD username ON user TYPE string ASSERT $value != NONE;
DEFINE INDEX user_username_index ON user COLUMNS username UNIQUE;
DEFINE FIELD email ON user TYPE string ASSERT $value != NONE;
DEFINE INDEX user_email_index ON user COLUMNS email UNIQUE;
DEFINE FIELD password ON user TYPE string ASSERT $value != NONE;
DEFINE FIELD registered_at ON user TYPE datetime ASSERT $value != NONE;
"
    );

    let events_dir = temp_dir.join("events");
    assert!(is_empty_folder(&events_dir)?);

    let migrations_dir = temp_dir.join("migrations");
    assert!(is_empty_folder(&migrations_dir)?);

    Ok(())
}

#[test]
fn scaffold_from_create_table_with_multi_column_unique_index() -> Result<()> {
    let temp_dir = TempDir::new()?;

    copy_folder(Path::new("schema-files"), &temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("scaffold")
        .arg("schema")
        .arg("schema-files/mssql/create_table_with_multi_column_unique_index.sql")
        .arg("--db-type")
        .arg("mssql");

    cmd.assert().success();

    let schemas_dir = temp_dir.join("schemas");

    assert!(schemas_dir.join("script_migration.surql").exists());

    let schema_files = std::fs::read_dir(&schemas_dir)?;
    assert_eq!(schema_files.count(), 2);

    let vote_schema = std::fs::read_to_string(schemas_dir.join("vote.surql"))?;
    assert_eq!(
        vote_schema,
        "DEFINE TABLE vote SCHEMALESS;

DEFINE FIELD id ON vote ASSERT $value != NONE;
DEFINE FIELD username ON vote TYPE string;
DEFINE FIELD movie ON vote TYPE string;
DEFINE INDEX Vote_Username_Movie_Unique ON vote COLUMNS username, movie UNIQUE;
"
    );

    let events_dir = temp_dir.join("events");
    assert!(is_empty_folder(&events_dir)?);

    let migrations_dir = temp_dir.join("migrations");
    assert!(is_empty_folder(&migrations_dir)?);

    Ok(())
}

#[test]
fn scaffold_from_create_table_with_index() -> Result<()> {
    let temp_dir = TempDir::new()?;

    copy_folder(Path::new("schema-files"), &temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("scaffold")
        .arg("schema")
        .arg("schema-files/mssql/create_table_with_index.sql")
        .arg("--db-type")
        .arg("mssql");

    cmd.assert().success();

    let schemas_dir = temp_dir.join("schemas");

    assert!(schemas_dir.join("script_migration.surql").exists());

    let schema_files = std::fs::read_dir(&schemas_dir)?;
    assert_eq!(schema_files.count(), 2);

    let daily_sales_schema = std::fs::read_to_string(schemas_dir.join("daily_sales.surql"))?;
    assert_eq!(
        daily_sales_schema,
        "DEFINE TABLE daily_sales SCHEMALESS;

DEFINE FIELD id ON daily_sales ASSERT $value != NONE;
DEFINE FIELD value ON daily_sales TYPE number ASSERT $value != NONE;
DEFINE FIELD date ON daily_sales TYPE datetime ASSERT $value != NONE;
DEFINE INDEX IX_DailySales_Sales ON daily_sales COLUMNS date;
"
    );

    let events_dir = temp_dir.join("events");
    assert!(is_empty_folder(&events_dir)?);

    let migrations_dir = temp_dir.join("migrations");
    assert!(is_empty_folder(&migrations_dir)?);

    Ok(())
}

#[test]
fn scaffold_from_create_table_with_multi_column_index() -> Result<()> {
    let temp_dir = TempDir::new()?;

    copy_folder(Path::new("schema-files"), &temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("scaffold")
        .arg("schema")
        .arg("schema-files/mssql/create_table_with_multi_column_index.sql")
        .arg("--db-type")
        .arg("mssql");

    cmd.assert().success();

    let schemas_dir = temp_dir.join("schemas");

    assert!(schemas_dir.join("script_migration.surql").exists());

    let schema_files = std::fs::read_dir(&schemas_dir)?;
    assert_eq!(schema_files.count(), 2);

    let product_schema = std::fs::read_to_string(schemas_dir.join("product.surql"))?;
    assert_eq!(
        product_schema,
        "DEFINE TABLE product SCHEMALESS;

DEFINE FIELD id ON product ASSERT $value != NONE;
DEFINE FIELD name ON product TYPE string;
DEFINE FIELD color ON product TYPE string;
DEFINE FIELD size ON product TYPE string;
DEFINE INDEX Vote_Name_Color_Size ON product COLUMNS name, color, size;
"
    );

    let events_dir = temp_dir.join("events");
    assert!(is_empty_folder(&events_dir)?);

    let migrations_dir = temp_dir.join("migrations");
    assert!(is_empty_folder(&migrations_dir)?);

    Ok(())
}

#[test]
fn scaffold_from_create_table_with_not_null_assert() -> Result<()> {
    let temp_dir = TempDir::new()?;

    copy_folder(Path::new("schema-files"), &temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("scaffold")
        .arg("schema")
        .arg("schema-files/mssql/create_table_with_not_null.sql")
        .arg("--db-type")
        .arg("mssql");

    cmd.assert().success();

    let schemas_dir = temp_dir.join("schemas");

    assert!(schemas_dir.join("script_migration.surql").exists());

    let schema_files = std::fs::read_dir(&schemas_dir)?;
    assert_eq!(schema_files.count(), 2);

    let post_schema = std::fs::read_to_string(schemas_dir.join("post.surql"))?;
    assert_eq!(
        post_schema,
        "DEFINE TABLE post SCHEMALESS;

DEFINE FIELD id ON post ASSERT $value != NONE;
DEFINE FIELD title ON post TYPE string ASSERT $value != NONE;
DEFINE FIELD content ON post TYPE string ASSERT $value != NONE;
DEFINE FIELD status ON post TYPE string ASSERT $value != NONE;
DEFINE FIELD created_at ON post TYPE datetime ASSERT $value != NONE;
"
    );

    let events_dir = temp_dir.join("events");
    assert!(is_empty_folder(&events_dir)?);

    let migrations_dir = temp_dir.join("migrations");
    assert!(is_empty_folder(&migrations_dir)?);

    Ok(())
}

#[test]
#[ignore]
fn scaffold_from_create_table_with_default_value() -> Result<()> {
    todo!();
}
