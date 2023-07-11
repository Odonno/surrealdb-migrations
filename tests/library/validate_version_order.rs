use anyhow::{ensure, Result};
use assert_fs::TempDir;
use regex::Regex;
use surrealdb_migrations::MigrationRunner;

use crate::helpers::*;

#[tokio::test]
async fn ok_if_no_migration_file() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name_in_dir(&temp_dir, &db_name)?;
    scaffold_empty_template(&temp_dir)?;

    let config_file_path = temp_dir.join(".surrealdb");

    let configuration = SurrealdbConfiguration {
        db: Some(db_name),
        ..Default::default()
    };

    let db = create_surrealdb_client(&configuration).await?;

    let runner =
        MigrationRunner::new(&db).use_config_file(config_file_path.to_str().unwrap_or_default());

    runner.validate_version_order().await?;

    Ok(())
}

#[tokio::test]
async fn ok_if_migrations_applied_but_no_new_migration() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name_in_dir(&temp_dir, &db_name)?;
    scaffold_blog_template(&temp_dir)?;

    let config_file_path = temp_dir.join(".surrealdb");

    let configuration = SurrealdbConfiguration {
        db: Some(db_name),
        ..Default::default()
    };

    let db = create_surrealdb_client(&configuration).await?;

    let runner =
        MigrationRunner::new(&db).use_config_file(config_file_path.to_str().unwrap_or_default());

    runner.up().await?;

    runner.validate_version_order().await?;

    Ok(())
}

#[tokio::test]
async fn ok_if_migrations_applied_with_new_migration_after_last_applied() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name_in_dir(&temp_dir, &db_name)?;
    scaffold_blog_template(&temp_dir)?;

    let config_file_path = temp_dir.join(".surrealdb");

    let configuration = SurrealdbConfiguration {
        db: Some(db_name),
        ..Default::default()
    };

    let db = create_surrealdb_client(&configuration).await?;

    let runner =
        MigrationRunner::new(&db).use_config_file(config_file_path.to_str().unwrap_or_default());

    let first_migration_name = get_first_migration_name(&temp_dir)?;
    runner.up_to(&first_migration_name).await?;

    runner.validate_version_order().await?;

    Ok(())
}

#[tokio::test]
async fn fails_if_migrations_applied_with_new_migration_before_last_applied() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name_in_dir(&temp_dir, &db_name)?;
    scaffold_blog_template(&temp_dir)?;

    let config_file_path = temp_dir.join(".surrealdb");

    let configuration = SurrealdbConfiguration {
        db: Some(db_name.to_string()),
        ..Default::default()
    };

    let db = create_surrealdb_client(&configuration).await?;

    let runner =
        MigrationRunner::new(&db).use_config_file(config_file_path.to_str().unwrap_or_default());

    let first_migration_file = get_first_migration_file(&temp_dir)?;
    std::fs::remove_file(first_migration_file)?;

    runner.up().await?;

    empty_folder(&temp_dir)?;

    add_migration_config_file_with_db_name_in_dir(&temp_dir, &db_name)?;
    scaffold_blog_template(&temp_dir)?;

    let result = runner.validate_version_order().await;

    ensure!(result.is_err());

    let error_regex =
        Regex::new(r"The following migrations have not been applied: \d+_\d+_AddAdminUser")?;

    let error_str = result.unwrap_err().to_string();
    let error_str = error_str.as_str();

    ensure!(error_regex.is_match(error_str));

    Ok(())
}

#[tokio::test]
async fn ok_if_migrations_applied_but_no_new_migration_with_inlined_down_files() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name_in_dir(&temp_dir, &db_name)?;
    scaffold_blog_template(&temp_dir)?;
    inline_down_migration_files(&temp_dir)?;

    let config_file_path = temp_dir.join(".surrealdb");

    let configuration = SurrealdbConfiguration {
        db: Some(db_name),
        ..Default::default()
    };

    let db = create_surrealdb_client(&configuration).await?;

    let runner =
        MigrationRunner::new(&db).use_config_file(config_file_path.to_str().unwrap_or_default());

    runner.up().await?;

    runner.validate_version_order().await?;

    Ok(())
}
