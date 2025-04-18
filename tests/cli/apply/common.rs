use assert_fs::TempDir;
use color_eyre::eyre::Result;
use predicates::prelude::*;
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

    cmd.assert().try_failure().and_then(|assert| {
        assert.try_stderr(predicate::str::contains(
            "You can\'t specify both `up` and `down` parameters at the same time",
        ))
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

    cmd.assert().try_failure().and_then(|assert| {
        assert.try_stderr(predicate::str::contains(
            "You can\'t specify both `up`, `down` and `reset` parameters at the same time",
        ))
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

    cmd.assert().try_failure().and_then(|assert| {
        assert.try_stderr(predicate::str::contains(
            "You can\'t specify both `up` and `reset` parameters at the same time",
        ))
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

    cmd.assert().try_failure().and_then(|assert| {
        assert.try_stderr(predicate::str::contains(
            "You can\'t specify both `down` and `reset` parameters at the same time",
        ))
    })?;

    temp_dir.close()?;

    Ok(())
}
