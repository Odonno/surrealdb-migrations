use std::collections::HashSet;

use assert_fs::TempDir;
use color_eyre::Result;
use surrealdb_migrations::MigrationRunner;

use crate::helpers::*;

#[tokio::test]
async fn apply_only_v1_schema() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name_in_dir(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;
    remove_folder(&temp_dir.join("migrations"))?;
    remove_folder(&temp_dir.join("events"))?;

    let schemas_dir = &temp_dir.join("schemas");
    let v1_dir = schemas_dir.join("v1");
    let v2_dir = schemas_dir.join("v2");

    create_folder(&v1_dir)?;
    create_folder(&v2_dir)?;

    move_file(&schemas_dir.join("post.surql"), &v1_dir.join("post.surql"))?;
    move_file(
        &schemas_dir.join("comment.surql"),
        &v2_dir.join("comment.surql"),
    )?;

    let config_file_path = temp_dir.join(".surrealdb");

    let configuration = SurrealdbConfiguration {
        db: Some(db_name),
        ..Default::default()
    };

    let db = create_surrealdb_client(&configuration).await?;

    let runner = MigrationRunner::new(&db).use_config_file(&config_file_path);

    runner.with_tags(&HashSet::from(["v1"])).up().await?;

    let post_table_exist = get_surrealdb_table_exists(Some(configuration.clone()), "post").await?;
    assert!(post_table_exist);

    let comment_table_exist = get_surrealdb_table_exists(Some(configuration), "comment").await?;
    assert!(!comment_table_exist);

    temp_dir.close()?;

    Ok(())
}

#[tokio::test]
async fn exclude_old_by_default() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name_in_dir(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;
    remove_folder(&temp_dir.join("migrations"))?;
    remove_folder(&temp_dir.join("events"))?;

    let schemas_dir = &temp_dir.join("schemas");
    let old_dir = schemas_dir.join("old");

    create_folder(&old_dir)?;

    move_file(
        &schemas_dir.join("comment.surql"),
        &old_dir.join("comment.surql"),
    )?;

    let config_file_path = temp_dir.join(".surrealdb");

    let configuration = SurrealdbConfiguration {
        db: Some(db_name),
        ..Default::default()
    };

    let db = create_surrealdb_client(&configuration).await?;

    let runner = MigrationRunner::new(&db).use_config_file(&config_file_path);

    runner.up().await?;

    let post_table_exist = get_surrealdb_table_exists(Some(configuration.clone()), "post").await?;
    assert!(post_table_exist);

    let comment_table_exist = get_surrealdb_table_exists(Some(configuration), "comment").await?;
    assert!(!comment_table_exist);

    temp_dir.close()?;

    Ok(())
}

#[tokio::test]
async fn exclude_v2_schema() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name_in_dir(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;
    remove_folder(&temp_dir.join("migrations"))?;
    remove_folder(&temp_dir.join("events"))?;

    let schemas_dir = &temp_dir.join("schemas");
    let v2_dir = schemas_dir.join("v2");

    create_folder(&v2_dir)?;

    move_file(
        &schemas_dir.join("comment.surql"),
        &v2_dir.join("comment.surql"),
    )?;
    let config_file_path = temp_dir.join(".surrealdb");

    let configuration = SurrealdbConfiguration {
        db: Some(db_name),
        ..Default::default()
    };

    let db = create_surrealdb_client(&configuration).await?;

    let runner = MigrationRunner::new(&db).use_config_file(&config_file_path);

    runner
        .with_exclude_tags(&HashSet::from(["v2"]))
        .up()
        .await?;

    let post_table_exist = get_surrealdb_table_exists(Some(configuration.clone()), "post").await?;
    assert!(post_table_exist);

    let comment_table_exist = get_surrealdb_table_exists(Some(configuration), "comment").await?;
    assert!(!comment_table_exist);

    temp_dir.close()?;

    Ok(())
}
