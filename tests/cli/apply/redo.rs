use assert_fs::TempDir;
use color_eyre::eyre::{ensure, Error, Result};
use insta::{assert_snapshot, Settings};

use crate::helpers::*;

#[test]
fn should_fail_to_redo_inexistant_migration() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;
    apply_migrations(&temp_dir, &db_name)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply").arg("--redo").arg("unknown-migration");

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
fn should_fail_to_redo_migration_that_was_not_applied() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;

    let first_migration_name = get_first_migration_name(&temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply").arg("--redo").arg(first_migration_name);

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

#[tokio::test]
async fn should_redo_migration_with_success() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;
    apply_migrations(&temp_dir, &db_name)?;

    let posts: Vec<Post> =
        get_surrealdb_records("test".to_string(), db_name.to_string(), "post".to_string()).await?;

    ensure!(posts.len() == 1, "There should be 1 post");

    let second_migration_name = get_second_migration_name(&temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply").arg("--redo").arg(second_migration_name);

    let assert = cmd.assert().try_success()?;
    let stdout = get_stdout_str(assert)?;

    let mut insta_settings = Settings::new();
    insta_settings.add_cli_location_filter();
    insta_settings.bind(|| {
        assert_snapshot!(stdout);
        Ok::<(), Error>(())
    })?;

    let posts: Vec<Post> =
        get_surrealdb_records("test".to_string(), db_name.to_string(), "post".to_string()).await?;

    ensure!(posts.len() == 2, "There should be 2 posts");

    temp_dir.close()?;

    Ok(())
}
