use assert_fs::TempDir;
use color_eyre::eyre::{Error, Result};
use insta::{assert_snapshot, Settings};

use crate::helpers::*;

#[test]
fn list_empty_migrations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_empty_template(&temp_dir, false)?;
    apply_migrations(&temp_dir, &db_name)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("list");

    cmd.assert()
        .try_success()
        .and_then(|assert| assert.try_stdout("No migrations applied yet!\n"))?;

    temp_dir.close()?;

    Ok(())
}

#[test]
fn list_blog_migrations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;
    apply_migrations(&temp_dir, &db_name)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("list").arg("--no-color");

    let assert = cmd.assert().try_success()?;
    let stdout = get_stdout_str(assert)?;

    let mut insta_settings = Settings::new();
    insta_settings.add_script_timestamp_filter();
    insta_settings.bind(|| {
        assert_snapshot!(stdout);
        Ok::<(), Error>(())
    })?;

    temp_dir.close()?;

    Ok(())
}
