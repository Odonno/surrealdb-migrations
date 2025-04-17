use assert_fs::TempDir;
use color_eyre::eyre::Result;
use surrealdb_migrations::MigrationRunner;

use crate::helpers::*;

#[tokio::test]
async fn ok_if_no_migration_file() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name_in_dir(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_empty_template(&temp_dir, true)?;

    let config_file_path = temp_dir.join(".surrealdb");

    let configuration = SurrealdbConfiguration {
        db: Some(db_name),
        ..Default::default()
    };

    let db = create_surrealdb_client(&configuration).await?;

    let runner = MigrationRunner::new(&db).use_config_file(&config_file_path);

    runner.validate_version_order().await?;

    temp_dir.close()?;

    Ok(())
}

#[tokio::test]
async fn ok_if_migrations_applied_but_no_new_migration() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name_in_dir(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, true)?;

    let config_file_path = temp_dir.join(".surrealdb");

    let configuration = SurrealdbConfiguration {
        db: Some(db_name),
        ..Default::default()
    };

    let db = create_surrealdb_client(&configuration).await?;

    let runner = MigrationRunner::new(&db).use_config_file(&config_file_path);

    runner.up().await?;

    runner.validate_version_order().await?;

    temp_dir.close()?;

    Ok(())
}

#[tokio::test]
async fn ok_if_migrations_applied_with_new_migration_after_last_applied() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name_in_dir(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, true)?;

    let config_file_path = temp_dir.join(".surrealdb");

    let configuration = SurrealdbConfiguration {
        db: Some(db_name),
        ..Default::default()
    };

    let db = create_surrealdb_client(&configuration).await?;

    let runner = MigrationRunner::new(&db).use_config_file(&config_file_path);

    let first_migration_name = get_first_migration_name(&temp_dir)?;
    runner.up_to(&first_migration_name).await?;

    runner.validate_version_order().await?;

    temp_dir.close()?;

    Ok(())
}

#[tokio::test]
async fn ok_if_migrations_applied_but_no_new_migration_with_inlined_down_files() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name_in_dir(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, true)?;
    inline_down_migration_files(&temp_dir)?;

    let config_file_path = temp_dir.join(".surrealdb");

    let configuration = SurrealdbConfiguration {
        db: Some(db_name),
        ..Default::default()
    };

    let db = create_surrealdb_client(&configuration).await?;

    let runner = MigrationRunner::new(&db).use_config_file(&config_file_path);

    runner.up().await?;

    runner.validate_version_order().await?;

    temp_dir.close()?;

    Ok(())
}
