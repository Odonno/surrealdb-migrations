use anyhow::Result;
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

    let building_schema = std::fs::read_to_string("tests-files/schemas/post.surql")?;

    assert_eq!(
        building_schema,
        r#"DEFINE TABLE post SCHEMALESS;

DEFINE FIELD id ON post;
DEFINE FIELD title ON post;
DEFINE FIELD content ON post;
DEFINE FIELD status ON post;
DEFINE FIELD created_at ON post;
"#
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

    let building_schema = std::fs::read_to_string("tests-files/schemas/Post.surql")?;

    assert_eq!(
        building_schema,
        r#"DEFINE TABLE Post SCHEMALESS;

DEFINE FIELD Id ON Post;
DEFINE FIELD Title ON Post;
DEFINE FIELD Content ON Post;
DEFINE FIELD Status ON Post;
DEFINE FIELD CreatedAt ON Post;
"#
    );

    let schema_files = std::fs::read_dir("tests-files/schemas")?;
    assert_eq!(schema_files.count(), 2);

    assert!(is_empty_folder("tests-files/events")?);
    assert!(is_empty_folder("tests-files/migrations")?);

    Ok(())
}
