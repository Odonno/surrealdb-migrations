use assert_fs::TempDir;
use color_eyre::Result;
use surrealdb_migrations::MigrationRunner;

use crate::helpers::*;

#[tokio::test]
async fn apply_initial_schema_changes() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name_in_dir(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;
    remove_folder(&temp_dir.join("migrations"))?;

    let config_file_path = temp_dir.join(".surrealdb");

    let configuration = SurrealdbConfiguration {
        db: Some(db_name),
        ..Default::default()
    };

    let db = create_surrealdb_client(&configuration).await?;

    let runner = MigrationRunner::new(&db).use_config_file(&config_file_path);

    runner.up().await?;

    temp_dir.close()?;

    Ok(())
}

#[tokio::test]
async fn apply_new_schema_changes() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name_in_dir(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;
    remove_folder(&temp_dir.join("migrations"))?;
    apply_migrations(&temp_dir, &db_name)?;
    add_category_schema_file(&temp_dir)?;

    let config_file_path = temp_dir.join(".surrealdb");

    let configuration = SurrealdbConfiguration {
        db: Some(db_name),
        ..Default::default()
    };

    let db = create_surrealdb_client(&configuration).await?;

    let runner = MigrationRunner::new(&db).use_config_file(&config_file_path);

    runner.up().await?;

    temp_dir.close()?;

    Ok(())
}

#[tokio::test]
async fn apply_initial_migrations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name_in_dir(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;

    let config_file_path = temp_dir.join(".surrealdb");

    let configuration = SurrealdbConfiguration {
        db: Some(db_name),
        ..Default::default()
    };

    let db = create_surrealdb_client(&configuration).await?;

    let runner = MigrationRunner::new(&db).use_config_file(&config_file_path);

    runner.up().await?;

    temp_dir.close()?;

    Ok(())
}

#[tokio::test]
async fn apply_new_migrations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name_in_dir(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;

    let first_migration_name = get_first_migration_name(&temp_dir)?;
    apply_migrations_up_to(&temp_dir, &db_name, &first_migration_name)?;

    let config_file_path = temp_dir.join(".surrealdb");

    let configuration = SurrealdbConfiguration {
        db: Some(db_name),
        ..Default::default()
    };

    let db = create_surrealdb_client(&configuration).await?;

    let runner = MigrationRunner::new(&db).use_config_file(&config_file_path);

    runner.up().await?;

    temp_dir.close()?;

    Ok(())
}

#[tokio::test]
async fn apply_with_db_configuration() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name_in_dir(&temp_dir, DbInstance::Admin, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;
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

    let runner = MigrationRunner::new(&db).use_config_file(&config_file_path);

    runner.up().await?;

    temp_dir.close()?;

    Ok(())
}

#[tokio::test]
async fn apply_should_skip_events_if_no_events_folder() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name_in_dir(&temp_dir, DbInstance::Admin, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;
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

    let runner = MigrationRunner::new(&db).use_config_file(&config_file_path);

    runner.up().await?;

    temp_dir.close()?;

    Ok(())
}

#[tokio::test]
async fn apply_with_inlined_down_files() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name_in_dir(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;
    inline_down_migration_files(&temp_dir)?;

    let config_file_path = temp_dir.join(".surrealdb");

    let configuration = SurrealdbConfiguration {
        db: Some(db_name),
        ..Default::default()
    };

    let db = create_surrealdb_client(&configuration).await?;

    let runner = MigrationRunner::new(&db).use_config_file(&config_file_path);

    runner.up().await?;

    temp_dir.close()?;

    Ok(())
}

#[tokio::test]
async fn apply_with_recursive_schema_folders() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name_in_dir(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;
    remove_folder(&temp_dir.join("events"))?;
    remove_folder(&temp_dir.join("migrations"))?;
    add_category_schema_file(&temp_dir)?;

    let schemas_path = &temp_dir.join("schemas");
    let v2_path = schemas_path.join("v2");
    let file_name = "category.surql";

    create_folder(&v2_path)?;
    move_file(&schemas_path.join(file_name), &v2_path.join(file_name))?;

    let config_file_path = temp_dir.join(".surrealdb");

    let configuration = SurrealdbConfiguration {
        db: Some(db_name),
        ..Default::default()
    };

    let db = create_surrealdb_client(&configuration).await?;

    let runner = MigrationRunner::new(&db).use_config_file(&config_file_path);

    runner.up().await?;

    temp_dir.close()?;

    Ok(())
}
