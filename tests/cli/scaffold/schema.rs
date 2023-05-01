use anyhow::Result;
use pretty_assertions::assert_eq;
use serial_test::serial;

use crate::helpers::common::*;

#[test]
#[serial]
fn scaffold_fails_from_empty_schema_file() -> Result<()> {
    clear_files_dir()?;

    let mut cmd = create_cmd()?;

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
#[serial]
fn scaffold_from_create_table() -> Result<()> {
    clear_files_dir()?;

    let mut cmd = create_cmd()?;

    cmd.arg("scaffold")
        .arg("schema")
        .arg("schema-files/mssql/create_table.sql")
        .arg("--db-type")
        .arg("mssql");

    cmd.assert().success();

    assert!(is_file_exists(
        "tests-files/schemas/script_migration.surql"
    )?);

    let post_schema = std::fs::read_to_string("tests-files/schemas/post.surql")?;

    assert_eq!(
        post_schema,
        "DEFINE TABLE post SCHEMALESS;

DEFINE FIELD id ON post;
DEFINE FIELD title ON post TYPE string;
DEFINE FIELD content ON post TYPE string;
DEFINE FIELD status ON post TYPE string;
DEFINE FIELD created_at ON post TYPE datetime;
"
    );

    let schema_files = std::fs::read_dir("tests-files/schemas")?;
    assert_eq!(schema_files.count(), 2);

    assert!(is_empty_folder("tests-files/events")?);
    assert!(is_empty_folder("tests-files/migrations")?);

    Ok(())
}

#[test]
#[serial]
fn scaffold_from_schema_file_but_preserve_casing() -> Result<()> {
    clear_files_dir()?;

    let mut cmd = create_cmd()?;

    cmd.arg("scaffold")
        .arg("schema")
        .arg("schema-files/mssql/create_table.sql")
        .arg("--db-type")
        .arg("mssql")
        .arg("--preserve-casing");

    cmd.assert().success();

    assert!(is_file_exists(
        "tests-files/schemas/script_migration.surql"
    )?);

    let post_schema = std::fs::read_to_string("tests-files/schemas/Post.surql")?;

    assert_eq!(
        post_schema,
        "DEFINE TABLE Post SCHEMALESS;

DEFINE FIELD Id ON Post;
DEFINE FIELD Title ON Post TYPE string;
DEFINE FIELD Content ON Post TYPE string;
DEFINE FIELD Status ON Post TYPE string;
DEFINE FIELD CreatedAt ON Post TYPE datetime;
"
    );

    let schema_files = std::fs::read_dir("tests-files/schemas")?;
    assert_eq!(schema_files.count(), 2);

    assert!(is_empty_folder("tests-files/events")?);
    assert!(is_empty_folder("tests-files/migrations")?);

    Ok(())
}

#[test]
#[serial]
fn scaffold_from_create_table_with_many_types() -> Result<()> {
    clear_files_dir()?;

    let mut cmd = create_cmd()?;

    cmd.arg("scaffold")
        .arg("schema")
        .arg("schema-files/mssql/create_table_with_many_types.sql")
        .arg("--db-type")
        .arg("mssql");

    cmd.assert().success();

    assert!(is_file_exists(
        "tests-files/schemas/script_migration.surql"
    )?);

    let test_schema = std::fs::read_to_string("tests-files/schemas/test.surql")?;

    assert_eq!(
        test_schema,
        "DEFINE TABLE test SCHEMALESS;

DEFINE FIELD id ON test;
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

    let schema_files = std::fs::read_dir("tests-files/schemas")?;
    assert_eq!(schema_files.count(), 2);

    assert!(is_empty_folder("tests-files/events")?);
    assert!(is_empty_folder("tests-files/migrations")?);

    Ok(())
}
