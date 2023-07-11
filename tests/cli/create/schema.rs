use anyhow::{ensure, Result};
use assert_fs::TempDir;
use pretty_assertions::assert_eq;

use crate::helpers::*;

#[test]
fn create_schema_file() -> Result<()> {
    let temp_dir = TempDir::new()?;

    scaffold_empty_template(&temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("create")
        .arg("schema")
        .arg("post")
        .arg("-f")
        .arg("name,title,published_at");

    cmd.assert().success();

    let post_file = std::fs::read_to_string(temp_dir.join("schemas/post.surql"))?;

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
fn create_schema_file_dry_run() -> Result<()> {
    let temp_dir = TempDir::new()?;

    let mut cmd = create_cmd(&temp_dir)?;

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

    let schemas_folder = temp_dir.join("schemas");
    assert_eq!(schemas_folder.exists(), false);

    Ok(())
}

#[test]
fn create_schemafull_table_file_from_config() -> Result<()> {
    let temp_dir = TempDir::new()?;

    add_migration_config_file_with_core_schema(&temp_dir, "full")?;
    scaffold_empty_template(&temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("create")
        .arg("schema")
        .arg("post")
        .arg("-f")
        .arg("name,title,published_at");

    cmd.assert().success();

    let post_file = std::fs::read_to_string(temp_dir.join("schemas/post.surql"))?;

    ensure!(
        post_file
            == "DEFINE TABLE post SCHEMAFULL;

DEFINE FIELD name ON post;
DEFINE FIELD title ON post;
DEFINE FIELD published_at ON post;"
    );

    Ok(())
}

#[test]
fn create_schemaless_table_file_from_invalid_config() -> Result<()> {
    let temp_dir = TempDir::new()?;

    add_migration_config_file_with_core_schema(&temp_dir, "invalid")?;
    scaffold_empty_template(&temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("create")
        .arg("schema")
        .arg("post")
        .arg("-f")
        .arg("name,title,published_at");

    cmd.assert().success();

    let post_file = std::fs::read_to_string(temp_dir.join("schemas/post.surql"))?;

    ensure!(
        post_file
            == "DEFINE TABLE post SCHEMALESS;

DEFINE FIELD name ON post;
DEFINE FIELD title ON post;
DEFINE FIELD published_at ON post;"
    );

    Ok(())
}

#[test]
fn create_schemafull_table_file_from_cli_arg() -> Result<()> {
    let temp_dir = TempDir::new()?;

    scaffold_empty_template(&temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("create")
        .arg("schema")
        .arg("post")
        .arg("-f")
        .arg("name,title,published_at")
        .arg("--schemafull");

    cmd.assert().success();

    let post_file = std::fs::read_to_string(temp_dir.join("schemas/post.surql"))?;

    assert_eq!(
        post_file,
        "DEFINE TABLE post SCHEMAFULL;

DEFINE FIELD name ON post;
DEFINE FIELD title ON post;
DEFINE FIELD published_at ON post;"
    );

    Ok(())
}
