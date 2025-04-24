use assert_fs::TempDir;
use color_eyre::Result;
use insta::assert_snapshot;
use surrealdb_migrations::MigrationRunner;

use crate::helpers::*;

#[tokio::test]
async fn should_fail_to_redo_inexistant_migration() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name_in_dir(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, true)?;
    apply_migrations(&temp_dir, &db_name)?;

    let config_file_path = temp_dir.join(".surrealdb");

    let configuration = SurrealdbConfiguration {
        db: Some(db_name),
        ..Default::default()
    };

    let db = create_surrealdb_client(&configuration).await?;

    let runner = MigrationRunner::new(&db).use_config_file(&config_file_path);

    let result = runner.redo("unknown-migration").await;

    assert!(result.is_err());
    assert_snapshot!(result.unwrap_err(), @"The migration file was not found");

    temp_dir.close()?;

    Ok(())
}

#[tokio::test]
async fn should_fail_to_redo_migration_that_was_not_applied() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name_in_dir(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, true)?;

    let first_migration_name = get_first_migration_name(&temp_dir)?;

    let config_file_path = temp_dir.join(".surrealdb");

    let configuration = SurrealdbConfiguration {
        db: Some(db_name),
        ..Default::default()
    };

    let db = create_surrealdb_client(&configuration).await?;

    let runner = MigrationRunner::new(&db).use_config_file(&config_file_path);

    let result = runner.redo(&first_migration_name).await;

    assert!(result.is_err());
    assert_snapshot!(result.unwrap_err(), @"This migration was not applied in the SurrealDB instance. Please make sure you correctly applied this migration before.");

    temp_dir.close()?;

    Ok(())
}

#[tokio::test]
async fn should_redo_migration_with_success() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name_in_dir(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, true)?;
    apply_migrations(&temp_dir, &db_name)?;

    let third_migration_name = get_third_migration_name(&temp_dir)?;

    let config_file_path = temp_dir.join(".surrealdb");

    let configuration = SurrealdbConfiguration {
        db: Some(db_name),
        ..Default::default()
    };

    let db = create_surrealdb_client(&configuration).await?;

    let runner = MigrationRunner::new(&db).use_config_file(&config_file_path);

    runner.redo(&third_migration_name).await?;

    temp_dir.close()?;

    Ok(())
}
