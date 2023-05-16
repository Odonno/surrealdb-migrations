use std::path::Path;

use anyhow::{ensure, Result};
use pretty_assertions::assert_eq;
use serial_test::serial;

use crate::helpers::*;

#[test]
#[serial]
fn create_schema_file() -> Result<()> {
    clear_tests_files()?;
    scaffold_empty_template()?;

    let mut cmd = create_cmd()?;

    cmd.arg("create")
        .arg("schema")
        .arg("post")
        .arg("-f")
        .arg("name,title,published_at");

    cmd.assert().success();

    let post_file = std::fs::read_to_string("tests-files/schemas/post.surql")?;

    assert_eq!(
        post_file,
        "DEFINE TABLE post SCHEMALESS;

DEFINE FIELD name ON post;
DEFINE FIELD title ON post;
DEFINE FIELD published_at ON post;"
    );

    Ok(())
}

#[test]
#[serial]
fn create_schema_file_dry_run() -> Result<()> {
    clear_tests_files()?;

    let mut cmd = create_cmd()?;

    cmd.arg("create")
        .arg("schema")
        .arg("post")
        .arg("-f")
        .arg("name,title,published_at")
        .arg("--dry-run");

    cmd.assert().success().stdout(
        "DEFINE TABLE post SCHEMALESS;

DEFINE FIELD name ON post;
DEFINE FIELD title ON post;
DEFINE FIELD published_at ON post;\n",
    );

    let schemas_folder = Path::new("tests-files/schemas");
    assert_eq!(schemas_folder.exists(), false);

    Ok(())
}

#[test]
#[serial]
fn create_schemafull_table_from_config() -> Result<()> {
    clear_tests_files()?;
    scaffold_empty_template()?;
    set_config_value("core", "schema", "full")?;

    let mut cmd = create_cmd()?;

    cmd.arg("create")
        .arg("schema")
        .arg("post")
        .arg("-f")
        .arg("name,title,published_at");

    cmd.assert().success();

    let post_file = std::fs::read_to_string("tests-files/schemas/post.surql")?;

    ensure!(
        post_file
            == "DEFINE TABLE post SCHEMAFULL;

DEFINE FIELD name ON post;
DEFINE FIELD title ON post;
DEFINE FIELD published_at ON post;"
    );

    reset_config()?;

    Ok(())
}

#[test]
#[serial]
fn create_schemaless_table_from_invalid_config() -> Result<()> {
    clear_tests_files()?;
    scaffold_empty_template()?;
    set_config_value("core", "schema", "invalid")?;

    let mut cmd = create_cmd()?;

    cmd.arg("create")
        .arg("schema")
        .arg("post")
        .arg("-f")
        .arg("name,title,published_at");

    cmd.assert().success();

    let post_file = std::fs::read_to_string("tests-files/schemas/post.surql")?;

    ensure!(
        post_file
            == "DEFINE TABLE post SCHEMALESS;

DEFINE FIELD name ON post;
DEFINE FIELD title ON post;
DEFINE FIELD published_at ON post;"
    );

    reset_config()?;

    Ok(())
}
