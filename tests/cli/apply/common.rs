use assert_fs::TempDir;
use color_eyre::eyre::{Error, Result};
use insta::{assert_snapshot, Settings};
use serial_test::serial;

use crate::helpers::*;

#[test]
#[serial]
fn apply_fails_if_both_up_and_down_args_provided() -> Result<()> {
    let temp_dir = TempDir::new()?;

    scaffold_blog_template(&temp_dir, false)?;

    let first_migration_name = get_first_migration_name(&temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply")
        .arg("--up")
        .arg(&first_migration_name)
        .arg("--down")
        .arg(&first_migration_name);

    let assert = cmd.assert().try_failure()?;
    let stderr = get_stderr_str(assert)?;

    let insta_settings = Settings::new();
    insta_settings.bind(|| {
        assert_snapshot!(stderr);
        Ok::<(), Error>(())
    })?;

    temp_dir.close()?;

    Ok(())
}

#[test]
#[serial]
fn apply_fails_if_both_up_and_down_and_reset_args_provided() -> Result<()> {
    let temp_dir = TempDir::new()?;

    scaffold_blog_template(&temp_dir, false)?;

    let first_migration_name = get_first_migration_name(&temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply")
        .arg("--up")
        .arg(&first_migration_name)
        .arg("--down")
        .arg(&first_migration_name)
        .arg("--reset");

    let assert = cmd.assert().try_failure()?;
    let stderr = get_stderr_str(assert)?;

    let insta_settings = Settings::new();
    insta_settings.bind(|| {
        assert_snapshot!(stderr);
        Ok::<(), Error>(())
    })?;

    temp_dir.close()?;

    Ok(())
}

#[test]
#[serial]
fn apply_fails_if_both_up_and_reset_args_provided() -> Result<()> {
    let temp_dir = TempDir::new()?;

    scaffold_blog_template(&temp_dir, false)?;

    let first_migration_name = get_first_migration_name(&temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply")
        .arg("--up")
        .arg(&first_migration_name)
        .arg("--reset");

    let assert = cmd.assert().try_failure()?;
    let stderr = get_stderr_str(assert)?;

    let insta_settings = Settings::new();
    insta_settings.bind(|| {
        assert_snapshot!(stderr);
        Ok::<(), Error>(())
    })?;

    temp_dir.close()?;

    Ok(())
}

#[test]
#[serial]
fn apply_fails_if_both_down_and_reset_args_provided() -> Result<()> {
    let temp_dir = TempDir::new()?;

    scaffold_blog_template(&temp_dir, false)?;

    let first_migration_name = get_first_migration_name(&temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply")
        .arg("--down")
        .arg(&first_migration_name)
        .arg("--reset");

    let assert = cmd.assert().try_failure()?;
    let stderr = get_stderr_str(assert)?;

    let insta_settings = Settings::new();
    insta_settings.bind(|| {
        assert_snapshot!(stderr);
        Ok::<(), Error>(())
    })?;

    temp_dir.close()?;

    Ok(())
}

#[test]
#[serial]
fn apply_fails_if_both_up_and_redo_args_provided() -> Result<()> {
    let temp_dir = TempDir::new()?;

    scaffold_blog_template(&temp_dir, false)?;

    let first_migration_name = get_first_migration_name(&temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply")
        .arg("--up")
        .arg(&first_migration_name)
        .arg("--redo")
        .arg(&first_migration_name);

    let assert = cmd.assert().try_failure()?;
    let stderr = get_stderr_str(assert)?;

    let insta_settings = Settings::new();
    insta_settings.bind(|| {
        assert_snapshot!(stderr);
        Ok::<(), Error>(())
    })?;

    temp_dir.close()?;

    Ok(())
}

#[test]
#[serial]
fn apply_fails_if_both_down_and_redo_args_provided() -> Result<()> {
    let temp_dir = TempDir::new()?;

    scaffold_blog_template(&temp_dir, false)?;

    let first_migration_name = get_first_migration_name(&temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply")
        .arg("--down")
        .arg(&first_migration_name)
        .arg("--redo")
        .arg(&first_migration_name);

    let assert = cmd.assert().try_failure()?;
    let stderr = get_stderr_str(assert)?;

    let insta_settings = Settings::new();
    insta_settings.bind(|| {
        assert_snapshot!(stderr);
        Ok::<(), Error>(())
    })?;

    temp_dir.close()?;

    Ok(())
}

#[test]
#[serial]
fn apply_fails_if_both_redo_and_reset_args_provided() -> Result<()> {
    let temp_dir = TempDir::new()?;

    scaffold_blog_template(&temp_dir, false)?;

    let first_migration_name = get_first_migration_name(&temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply")
        .arg("--redo")
        .arg(&first_migration_name)
        .arg("--reset");

    let assert = cmd.assert().try_failure()?;
    let stderr = get_stderr_str(assert)?;

    let insta_settings = Settings::new();
    insta_settings.bind(|| {
        assert_snapshot!(stderr);
        Ok::<(), Error>(())
    })?;

    temp_dir.close()?;

    Ok(())
}

#[test]
#[serial]
fn apply_fails_if_both_up_and_down_redo_and_reset_args_provided() -> Result<()> {
    let temp_dir = TempDir::new()?;

    scaffold_blog_template(&temp_dir, false)?;

    let first_migration_name = get_first_migration_name(&temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply")
        .arg("--up")
        .arg(&first_migration_name)
        .arg("--down")
        .arg(&first_migration_name)
        .arg("--redo")
        .arg(&first_migration_name)
        .arg("--reset");

    let assert = cmd.assert().try_failure()?;
    let stderr = get_stderr_str(assert)?;

    let insta_settings = Settings::new();
    insta_settings.bind(|| {
        assert_snapshot!(stderr);
        Ok::<(), Error>(())
    })?;

    temp_dir.close()?;

    Ok(())
}
