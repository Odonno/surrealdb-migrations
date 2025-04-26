use assert_fs::TempDir;
use color_eyre::eyre::{ensure, Result};
use insta::{assert_snapshot, Settings};

use crate::helpers::*;

#[test]
fn apply_initial_migrations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    println!("{}", db_name);

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, true)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply");

    cmd.assert().try_success().and_then(|assert| {
        assert.try_stdout(
            "Executing migration Initial...
Executing migration AddAdminUser...
Executing migration AddPost...
Executing migration CommentPost...
Migration files successfully executed!\n",
        )
    })?;

    temp_dir.close()?;

    Ok(())
}

#[test]
fn apply_new_migrations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, true)?;

    let second_migration_name = get_second_migration_name(&temp_dir)?;
    apply_migrations_up_to(&temp_dir, &db_name, &second_migration_name)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply");

    cmd.assert().try_success().and_then(|assert| {
        assert.try_stdout(
            "Executing migration AddPost...
Executing migration CommentPost...
Migration files successfully executed!\n",
        )
    })?;

    temp_dir.close()?;

    Ok(())
}

#[tokio::test]
#[ignore = "should 'dry-run' run against a concrete database?"]
async fn apply_initial_migrations_in_dry_run() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, true)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply").arg("--dry-run");

    cmd.assert()
        .try_success()
        .and_then(|assert| assert.try_stdout(""))?;

    let is_empty = is_surreal_db_empty(None, Some(db_name)).await?;
    ensure!(is_empty, "SurrealDB should be empty");

    temp_dir.close()?;

    Ok(())
}

#[test]
fn apply_with_inlined_down_files() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, true)?;
    inline_down_migration_files(&temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply");

    cmd.assert().try_success().and_then(|assert| {
        assert.try_stdout(
            "Executing migration Initial...
Executing migration AddAdminUser...
Executing migration AddPost...
Executing migration CommentPost...
Migration files successfully executed!\n",
        )
    })?;

    temp_dir.close()?;

    Ok(())
}

#[test]
#[ignore = "should 'dry-run' run against a concrete database?"]
fn apply_and_output_initial_migrations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, true)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply").arg("--dry-run").arg("-o");

    cmd.assert().try_success().map(|assert| {
        let out = String::from_utf8_lossy(&assert.get_output().stdout);

        let mut settings = Settings::new();
        settings.add_script_timestamp_filter();

        settings.bind(|| {
            assert_snapshot!(out);
        });
    })?;

    temp_dir.close()?;

    Ok(())
}

#[test]
fn apply_and_output_new_migrations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, true)?;

    let first_migration_name = get_first_migration_name(&temp_dir)?;
    apply_migrations_up_to(&temp_dir, &db_name, &first_migration_name)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply").arg("--dry-run").arg("--output");

    cmd.assert().try_success().map(|assert| {
        let out = String::from_utf8_lossy(&assert.get_output().stdout);

        let mut settings = Settings::new();
        settings.add_script_timestamp_filter();

        settings.bind(|| {
            assert_snapshot!(out);
        });
    })?;

    temp_dir.close()?;

    Ok(())
}
