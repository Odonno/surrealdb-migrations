use anyhow::Result;
use serial_test::serial;

use crate::helpers::common::*;

#[test]
#[serial]
fn create_schema_file() -> Result<()> {
    clear_files_dir()?;
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
fn create_event_file() -> Result<()> {
    clear_files_dir()?;
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

DEFINE EVENT publish_post ON TABLE publish_post WHEN $before == NONE THEN (
    # TODO
);",
    );

    Ok(())
}

#[test]
#[serial]
fn create_migration_file() -> Result<()> {
    clear_files_dir()?;
    scaffold_empty_template()?;

    let mut cmd = create_cmd()?;

    cmd.arg("create").arg("migration").arg("AddPost");

    cmd.assert().success();

    let migrations_folder = std::fs::read_dir("tests-files/migrations")?;

    assert_eq!(migrations_folder.count(), 1);

    Ok(())
}

#[test]
#[serial]
fn create_schema_file_dry_run() -> Result<()> {
    clear_files_dir()?;

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

    Ok(())
}

#[test]
#[serial]
fn create_event_file_dry_run() -> Result<()> {
    clear_files_dir()?;

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

DEFINE EVENT publish_post ON TABLE publish_post WHEN $before == NONE THEN (
    # TODO
);\n",
    );

    Ok(())
}
