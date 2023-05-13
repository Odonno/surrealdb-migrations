use anyhow::Result;
use serial_test::serial;

use crate::helpers::*;

#[test]
#[serial]
fn scaffold_empty_template() -> Result<()> {
    clear_tests_files()?;

    let mut cmd = create_cmd()?;

    cmd.arg("scaffold").arg("template").arg("empty");

    cmd.assert().success();

    assert!(are_folders_equivalent(
        "templates/empty/schemas",
        "tests-files/schemas"
    )?);

    assert!(is_empty_folder("tests-files/events")?);
    assert!(is_empty_folder("tests-files/migrations")?);

    Ok(())
}

#[test]
#[serial]
fn scaffold_blog_template() -> Result<()> {
    clear_tests_files()?;

    let mut cmd = create_cmd()?;

    cmd.arg("scaffold").arg("template").arg("blog");

    cmd.assert().success();

    assert!(are_folders_equivalent(
        "templates/blog/schemas",
        "tests-files/schemas"
    )?);

    assert!(are_folders_equivalent(
        "templates/blog/events",
        "tests-files/events"
    )?);

    let migration_files = std::fs::read_dir("tests-files/migrations")?;
    assert_eq!(migration_files.count(), 3);

    Ok(())
}

#[test]
#[serial]
fn scaffold_ecommerce_template() -> Result<()> {
    clear_tests_files()?;

    let mut cmd = create_cmd()?;

    cmd.arg("scaffold").arg("template").arg("ecommerce");

    cmd.assert().success();

    assert!(are_folders_equivalent(
        "templates/ecommerce/schemas",
        "tests-files/schemas"
    )?);

    assert!(are_folders_equivalent(
        "templates/ecommerce/events",
        "tests-files/events"
    )?);

    let migration_files = std::fs::read_dir("tests-files/migrations")?;
    assert_eq!(migration_files.count(), 3);

    Ok(())
}

#[test]
#[serial]
fn scaffold_fails_if_schemas_folder_already_exists() -> Result<()> {
    clear_tests_files()?;

    fs_extra::dir::create("tests-files", false)?;
    fs_extra::dir::create("tests-files/schemas", false)?;

    let mut cmd = create_cmd()?;

    cmd.arg("scaffold").arg("template").arg("blog");

    cmd.assert()
        .failure()
        .stderr("Error: 'schemas' folder already exists.\n");

    Ok(())
}

#[test]
#[serial]
fn scaffold_fails_if_events_folder_already_exists() -> Result<()> {
    clear_tests_files()?;

    fs_extra::dir::create("tests-files", false)?;
    fs_extra::dir::create("tests-files/events", false)?;

    let mut cmd = create_cmd()?;

    cmd.arg("scaffold").arg("template").arg("blog");

    cmd.assert()
        .failure()
        .stderr("Error: 'events' folder already exists.\n");

    Ok(())
}

#[test]
#[serial]
fn scaffold_fails_if_migrations_folder_already_exists() -> Result<()> {
    clear_tests_files()?;

    fs_extra::dir::create("tests-files", false)?;
    fs_extra::dir::create("tests-files/migrations", false)?;

    let mut cmd = create_cmd()?;

    cmd.arg("scaffold").arg("template").arg("blog");

    cmd.assert()
        .failure()
        .stderr("Error: 'migrations' folder already exists.\n");

    Ok(())
}

#[test]
#[serial]
fn scaffold_fails_if_invalid_template_name() -> Result<()> {
    clear_tests_files()?;

    let mut cmd = create_cmd()?;

    cmd.arg("scaffold").arg("template").arg("invalid");

    cmd.assert().failure();

    Ok(())
}
