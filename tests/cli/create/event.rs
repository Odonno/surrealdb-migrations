use anyhow::{ensure, Result};
use pretty_assertions::assert_eq;
use serial_test::serial;
use std::path::Path;

use crate::helpers::*;

#[test]
#[serial]
fn create_event_file() -> Result<()> {
    clear_tests_files()?;
    scaffold_empty_template()?;

    let mut cmd = create_cmd()?;

    cmd.arg("create")
        .arg("event")
        .arg("publish_post")
        .arg("-f")
        .arg("post_id,created_at");

    cmd.assert().success();

    let publish_post_file = std::fs::read_to_string("tests-files/events/publish_post.surql")?;

    assert_eq!(
        publish_post_file,
        "DEFINE TABLE publish_post SCHEMALESS;

DEFINE FIELD post_id ON publish_post;
DEFINE FIELD created_at ON publish_post;

DEFINE EVENT publish_post ON TABLE publish_post WHEN $event == "CREATE" THEN (
    # TODO
);",
    );

    Ok(())
}

#[test]
#[serial]
fn create_event_file_dry_run() -> Result<()> {
    clear_tests_files()?;

    let mut cmd = create_cmd()?;

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

DEFINE EVENT publish_post ON TABLE publish_post WHEN $event == "CREATE" THEN (
    # TODO
);\n",
    );

    let events_folder = Path::new("tests-files/events");
    assert_eq!(events_folder.exists(), false);

    Ok(())
}

#[test]
#[serial]
fn create_event_file_with_schemafull_table_from_config() -> Result<()> {
    clear_tests_files()?;
    scaffold_empty_template()?;
    set_config_value("core", "schema", "full")?;

    let mut cmd = create_cmd()?;

    cmd.arg("create")
        .arg("event")
        .arg("publish_post")
        .arg("-f")
        .arg("post_id,created_at");

    cmd.assert().success();

    let publish_post_file = std::fs::read_to_string("tests-files/events/publish_post.surql")?;

    ensure!(
        publish_post_file
            == "DEFINE TABLE publish_post SCHEMAFULL;

DEFINE FIELD post_id ON publish_post;
DEFINE FIELD created_at ON publish_post;

DEFINE EVENT publish_post ON TABLE publish_post WHEN $event == "CREATE" THEN (
    # TODO
);",
    );

    reset_config()?;

    Ok(())
}

#[test]
#[serial]
fn create_event_file_with_schemaless_table_from_invalid_config() -> Result<()> {
    clear_tests_files()?;
    scaffold_empty_template()?;
    set_config_value("core", "schema", "invalid")?;

    let mut cmd = create_cmd()?;

    cmd.arg("create")
        .arg("event")
        .arg("publish_post")
        .arg("-f")
        .arg("post_id,created_at");

    cmd.assert().success();

    let publish_post_file = std::fs::read_to_string("tests-files/events/publish_post.surql")?;

    ensure!(
        publish_post_file
            == "DEFINE TABLE publish_post SCHEMALESS;

DEFINE FIELD post_id ON publish_post;
DEFINE FIELD created_at ON publish_post;

DEFINE EVENT publish_post ON TABLE publish_post WHEN $event == "CREATE" THEN (
    # TODO
);",
    );

    reset_config()?;

    Ok(())
}

#[test]
#[serial]
fn create_event_file_with_schemafull_table_from_cli_arg() -> Result<()> {
    clear_tests_files()?;
    scaffold_empty_template()?;

    let mut cmd = create_cmd()?;

    cmd.arg("create")
        .arg("event")
        .arg("publish_post")
        .arg("-f")
        .arg("post_id,created_at")
        .arg("--schemafull");

    cmd.assert().success();

    let publish_post_file = std::fs::read_to_string("tests-files/events/publish_post.surql")?;

    assert_eq!(
        publish_post_file,
        "DEFINE TABLE publish_post SCHEMAFULL;

DEFINE FIELD post_id ON publish_post;
DEFINE FIELD created_at ON publish_post;

DEFINE EVENT publish_post ON TABLE publish_post WHEN $event == "CREATE" THEN (
    # TODO
);",
    );

    Ok(())
}
