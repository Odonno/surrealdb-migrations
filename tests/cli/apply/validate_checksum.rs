use assert_fs::TempDir;
use color_eyre::eyre::{Error, Result};
use insta::{assert_snapshot, Settings};

use crate::helpers::*;

#[test]
fn ok_if_no_changes() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;
    apply_migrations(&temp_dir, &db_name)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply").arg("--validate-checksum");

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
fn fails_if_migration_file_is_missing() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;
    apply_migrations(&temp_dir, &db_name)?;

    let first_migration_file = get_first_migration_file(&temp_dir)?;
    std::fs::remove_file(first_migration_file)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply").arg("--validate-checksum");

    let assert = cmd.assert().try_failure()?;
    let stderr = get_stderr_str(assert)?;

    let mut insta_settings = Settings::new();
    insta_settings.add_cli_location_filter();
    insta_settings.add_script_timestamp_filter();
    insta_settings.bind(|| {
        assert_snapshot!(stderr);
        Ok::<(), Error>(())
    })?;

    temp_dir.close()?;

    Ok(())
}

#[test]
fn fails_if_migration_file_content_changed() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;
    apply_migrations(&temp_dir, &db_name)?;

    let first_migration_file = get_first_migration_file(&temp_dir)?;
    std::fs::write(
        first_migration_file,
        "CREATE permission:new SET name = 'new';",
    )?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply").arg("--validate-checksum");

    let assert = cmd.assert().try_failure()?;
    let stderr = get_stderr_str(assert)?;

    let mut insta_settings = Settings::new();
    insta_settings.add_cli_location_filter();
    insta_settings.add_script_timestamp_filter();
    insta_settings.bind(|| {
        assert_snapshot!(stderr);
        Ok::<(), Error>(())
    })?;

    temp_dir.close()?;

    Ok(())
}
