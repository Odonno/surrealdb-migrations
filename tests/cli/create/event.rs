use anyhow::{ensure, Result};
use assert_fs::TempDir;
use pretty_assertions::assert_eq;

use crate::helpers::*;

#[test]
fn create_event_file() -> Result<()> {
    let temp_dir = TempDir::new()?;

    scaffold_empty_template(&temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("create")
        .arg("event")
        .arg("publish_post")
        .arg("-f")
        .arg("post_id,created_at");

    cmd.assert().success();

    let publish_post_file = std::fs::read_to_string(temp_dir.join("events/publish_post.surql"))?;

    assert_eq!(
        publish_post_file,
        "DEFINE TABLE publish_post SCHEMALESS;

DEFINE FIELD post_id ON publish_post;
DEFINE FIELD created_at ON publish_post;

DEFINE EVENT publish_post ON TABLE publish_post WHEN $before == NONE THEN (
    # TODO
);",
    );

    Ok(())
}

#[test]
fn create_event_file_dry_run() -> Result<()> {
    let temp_dir = TempDir::new()?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("create")
        .arg("event")
        .arg("publish_post")
        .arg("-f")
        .arg("post_id,created_at")
        .arg("--dry-run");

    cmd.assert().success().stdout(
        "DEFINE TABLE publish_post SCHEMALESS;

DEFINE FIELD post_id ON publish_post;
DEFINE FIELD created_at ON publish_post;

DEFINE EVENT publish_post ON TABLE publish_post WHEN $before == NONE THEN (
    # TODO
);\n",
    );

    let events_folder = temp_dir.join("events");
    assert_eq!(events_folder.exists(), false);

    Ok(())
}

#[test]
fn create_event_file_with_schemafull_table_from_config() -> Result<()> {
    let temp_dir = TempDir::new()?;

    add_migration_config_file_with_core_schema(&temp_dir, "full")?;
    scaffold_empty_template(&temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("create")
        .arg("event")
        .arg("publish_post")
        .arg("-f")
        .arg("post_id,created_at");

    cmd.assert().success();

    let publish_post_file = std::fs::read_to_string(temp_dir.join("events/publish_post.surql"))?;

    ensure!(
        publish_post_file
            == "DEFINE TABLE publish_post SCHEMAFULL;

DEFINE FIELD post_id ON publish_post;
DEFINE FIELD created_at ON publish_post;

DEFINE EVENT publish_post ON TABLE publish_post WHEN $before == NONE THEN (
    # TODO
);",
    );

    Ok(())
}

#[test]
fn create_event_file_with_schemaless_table_from_invalid_config() -> Result<()> {
    let temp_dir = TempDir::new()?;

    add_migration_config_file_with_core_schema(&temp_dir, "invalid")?;
    scaffold_empty_template(&temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("create")
        .arg("event")
        .arg("publish_post")
        .arg("-f")
        .arg("post_id,created_at");

    cmd.assert().success();

    let publish_post_file = std::fs::read_to_string(temp_dir.join("events/publish_post.surql"))?;

    ensure!(
        publish_post_file
            == "DEFINE TABLE publish_post SCHEMALESS;

DEFINE FIELD post_id ON publish_post;
DEFINE FIELD created_at ON publish_post;

DEFINE EVENT publish_post ON TABLE publish_post WHEN $before == NONE THEN (
    # TODO
);",
    );

    Ok(())
}

#[test]
fn create_event_file_with_schemafull_table_from_cli_arg() -> Result<()> {
    let temp_dir = TempDir::new()?;

    scaffold_empty_template(&temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("create")
        .arg("event")
        .arg("publish_post")
        .arg("-f")
        .arg("post_id,created_at")
        .arg("--schemafull");

    cmd.assert().success();

    let publish_post_file = std::fs::read_to_string(temp_dir.join("events/publish_post.surql"))?;

    assert_eq!(
        publish_post_file,
        "DEFINE TABLE publish_post SCHEMAFULL;

DEFINE FIELD post_id ON publish_post;
DEFINE FIELD created_at ON publish_post;

DEFINE EVENT publish_post ON TABLE publish_post WHEN $before == NONE THEN (
    # TODO
);",
    );

    Ok(())
}
