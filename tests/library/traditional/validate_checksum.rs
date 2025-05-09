use assert_fs::TempDir;
use color_eyre::eyre::{Error, Result};
use insta::{assert_snapshot, Settings};
use surrealdb_migrations::MigrationRunner;

use crate::helpers::*;

#[tokio::test]
async fn ok_if_no_changes() -> Result<()> {
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

    runner.validate_checksum().await?;

    temp_dir.close()?;

    Ok(())
}

#[tokio::test]
async fn fails_if_migration_file_is_missing() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name_in_dir(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, true)?;
    apply_migrations(&temp_dir, &db_name)?;

    let second_migration_file = get_second_migration_file(&temp_dir)?;
    std::fs::remove_file(second_migration_file)?;

    let config_file_path = temp_dir.join(".surrealdb");

    let configuration = SurrealdbConfiguration {
        db: Some(db_name),
        ..Default::default()
    };

    let db = create_surrealdb_client(&configuration).await?;

    let runner = MigrationRunner::new(&db).use_config_file(&config_file_path);

    let result = runner.validate_checksum().await;

    assert!(result.is_err());

    let mut insta_settings = Settings::new();
    insta_settings.add_script_timestamp_filter();
    insta_settings.bind(|| {
        assert_snapshot!(result.unwrap_err(), @"The migration file '[timestamp]_AddAdminUser' does not exist.");
        Ok::<(), Error>(())
    })?;

    temp_dir.close()?;

    Ok(())
}

#[tokio::test]
async fn fails_if_migration_file_content_changed() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name_in_dir(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, true)?;
    apply_migrations(&temp_dir, &db_name)?;

    let second_migration_file = get_second_migration_file(&temp_dir)?;
    std::fs::write(
        second_migration_file,
        "CREATE permission:new SET name = 'new';",
    )?;

    let config_file_path = temp_dir.join(".surrealdb");

    let configuration = SurrealdbConfiguration {
        db: Some(db_name),
        ..Default::default()
    };

    let db = create_surrealdb_client(&configuration).await?;

    let runner = MigrationRunner::new(&db).use_config_file(&config_file_path);

    let result = runner.validate_checksum().await;

    assert!(result.is_err());

    let mut insta_settings = Settings::new();
    insta_settings.add_script_timestamp_filter();
    insta_settings.bind(|| {
        assert_snapshot!(result.unwrap_err(), @"The checksum does not match for migration '[timestamp]_AddAdminUser'.");
        Ok::<(), Error>(())
    })?;

    temp_dir.close()?;

    Ok(())
}
