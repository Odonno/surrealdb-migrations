use anyhow::Result;
use assert_fs::TempDir;
use surrealdb_migrations::MigrationRunner;

use crate::helpers::*;

#[tokio::test]
async fn apply_initial_schema_changes() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name_in_dir(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir)?;
    remove_folder(&temp_dir.join("migrations"))?;

    let config_file_path = temp_dir.join(".surrealdb");

    let configuration = SurrealdbConfiguration {
        db: Some(db_name),
        ..Default::default()
    };

    let db = create_surrealdb_client(&configuration).await?;

    let runner =
        MigrationRunner::new(&db).use_config_file(config_file_path.to_str().unwrap_or_default());

    runner.up().await?;

    Ok(())
}

#[tokio::test]
async fn apply_new_schema_changes() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name_in_dir(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir)?;
    remove_folder(&temp_dir.join("migrations"))?;
    apply_migrations(&temp_dir, &db_name)?;
    add_category_schema_file(&temp_dir)?;

    let config_file_path = temp_dir.join(".surrealdb");

    let configuration = SurrealdbConfiguration {
        db: Some(db_name),
        ..Default::default()
    };

    let db = create_surrealdb_client(&configuration).await?;

    let runner =
        MigrationRunner::new(&db).use_config_file(config_file_path.to_str().unwrap_or_default());

    runner.up().await?;

    Ok(())
}

#[tokio::test]
async fn apply_initial_migrations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name_in_dir(&temp_dir, DbInstance::Root, &db_name)?;
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

    Ok(())
}

#[tokio::test]
async fn apply_new_migrations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name_in_dir(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir)?;

    let first_migration_name = get_first_migration_name(&temp_dir)?;
    apply_migrations_up_to(&temp_dir, &db_name, &first_migration_name)?;

    let config_file_path = temp_dir.join(".surrealdb");

    let configuration = SurrealdbConfiguration {
        db: Some(db_name),
        ..Default::default()
    };

    let db = create_surrealdb_client(&configuration).await?;

    let runner =
        MigrationRunner::new(&db).use_config_file(config_file_path.to_str().unwrap_or_default());

    runner.up().await?;

    Ok(())
}

#[tokio::test]
async fn apply_with_db_configuration() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name_in_dir(&temp_dir, DbInstance::Admin, &db_name)?;
    scaffold_blog_template(&temp_dir)?;
    empty_folder(&temp_dir.join("migrations"))?;

    let config_file_path = temp_dir.join(".surrealdb");

    let configuration = SurrealdbConfiguration {
        address: Some("ws://localhost:8001".to_string()),
        url: None,
        username: Some("admin".to_string()),
        password: Some("admin".to_string()),
        ns: Some("test".to_string()),
        db: Some(db_name),
    };
    let db = create_surrealdb_client(&configuration).await?;

    let runner =
        MigrationRunner::new(&db).use_config_file(config_file_path.to_str().unwrap_or_default());

    runner.up().await?;

    Ok(())
}

#[tokio::test]
async fn apply_should_skip_events_if_no_events_folder() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name_in_dir(&temp_dir, DbInstance::Admin, &db_name)?;
    scaffold_blog_template(&temp_dir)?;
    empty_folder(&temp_dir.join("migrations"))?;
    remove_folder(&temp_dir.join("events"))?;

    let config_file_path = temp_dir.join(".surrealdb");

    let configuration = SurrealdbConfiguration {
        address: Some("ws://localhost:8001".to_string()),
        url: None,
        username: Some("admin".to_string()),
        password: Some("admin".to_string()),
        ns: Some("namespace".to_string()),
        db: Some("database".to_string()),
    };
    let db = create_surrealdb_client(&configuration).await?;

    let runner =
        MigrationRunner::new(&db).use_config_file(config_file_path.to_str().unwrap_or_default());

    runner.up().await?;

    Ok(())
}

#[tokio::test]
async fn apply_with_inlined_down_files() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name_in_dir(&temp_dir, DbInstance::Root, &db_name)?;
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

    Ok(())
}
