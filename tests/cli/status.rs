use std::fs;

use assert_fs::TempDir;
use color_eyre::eyre::{Error, Result};
use insta::{assert_snapshot, Settings};

use crate::helpers::*;

#[test]
fn fails_if_no_script_migration_definition() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("status");

    let assert = cmd.assert().try_failure()?;
    let stderr = get_stderr_str(assert)?;

    let mut insta_settings = Settings::new();
    insta_settings.add_cli_location_filter();
    insta_settings.bind(|| {
        assert_snapshot!(stderr);
        Ok::<(), Error>(())
    })?;

    temp_dir.close()?;

    Ok(())
}

#[test]
fn status_up_to_date() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;
    apply_migrations(&temp_dir, &db_name)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("status");

    let assert = cmd.assert().try_success()?;
    let stdout = get_stdout_str(assert)?;

    let mut insta_settings = Settings::new();
    insta_settings.add_cli_location_filter();
    insta_settings.bind(|| {
        assert_snapshot!(stdout);
        Ok::<(), Error>(())
    })?;

    temp_dir.close()?;

    Ok(())
}

#[test]
fn checksum_capability_disabled() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;
    disable_checksum_capability(&temp_dir)?;
    apply_migrations(&temp_dir, &db_name)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("status");

    let assert = cmd.assert().try_success()?;
    let stdout = get_stdout_str(assert)?;

    let mut insta_settings = Settings::new();
    insta_settings.add_cli_location_filter();
    insta_settings.bind(|| {
        assert_snapshot!(stdout);
        Ok::<(), Error>(())
    })?;

    temp_dir.close()?;

    Ok(())
}

#[test]
fn no_migrations_applied() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_empty_template(&temp_dir, false)?;
    apply_migrations(&temp_dir, &db_name)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("status");

    let assert = cmd.assert().try_success()?;
    let stdout = get_stdout_str(assert)?;

    let mut insta_settings = Settings::new();
    insta_settings.add_cli_location_filter();
    insta_settings.bind(|| {
        assert_snapshot!(stdout);
        Ok::<(), Error>(())
    })?;

    temp_dir.close()?;

    Ok(())
}

#[test]
fn with_migrations_left() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;
    apply_migrations(&temp_dir, &db_name)?;
    apply_migrations_down_single(&temp_dir, &db_name)?;
    apply_migrations_down_single(&temp_dir, &db_name)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("status");

    let assert = cmd.assert().try_success()?;
    let stdout = get_stdout_str(assert)?;

    let mut insta_settings = Settings::new();
    insta_settings.add_cli_location_filter();
    insta_settings.bind(|| {
        assert_snapshot!(stdout);
        Ok::<(), Error>(())
    })?;

    temp_dir.close()?;

    Ok(())
}

#[test]
fn with_missing_migration_files() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;
    apply_migrations(&temp_dir, &db_name)?;

    let second_migration_file = get_second_migration_file(&temp_dir)?;
    fs::remove_file(&second_migration_file)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("status");

    let assert = cmd.assert().try_success()?;
    let stdout = get_stdout_str(assert)?;

    let mut insta_settings = Settings::new();
    insta_settings.add_cli_location_filter();
    insta_settings.add_script_timestamp_filter();
    insta_settings.bind(|| {
        assert_snapshot!(stdout);
        Ok::<(), Error>(())
    })?;

    temp_dir.close()?;

    Ok(())
}
