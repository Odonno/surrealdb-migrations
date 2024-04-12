use assert_fs::TempDir;
use chrono::{DateTime, Local};
use color_eyre::{
    eyre::{ensure, Context, ContextCompat},
    Result,
};
use surrealdb_migrations::MigrationRunner;

use crate::helpers::*;

#[tokio::test]
async fn list_empty_migrations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name_in_dir(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_empty_template(&temp_dir)?;
    apply_migrations(&temp_dir, &db_name)?;

    let config_file_path = temp_dir.join(".surrealdb");

    let configuration = SurrealdbConfiguration {
        db: Some(db_name),
        ..Default::default()
    };

    let db = create_surrealdb_client(&configuration).await?;

    let runner = MigrationRunner::new(&db).use_config_file(&config_file_path);

    let migrations_applied = runner.list().await?;

    ensure!(
        migrations_applied.is_empty(),
        "Expected no migrations to be applied"
    );

    Ok(())
}

#[tokio::test]
async fn list_blog_migrations() -> Result<()> {
    let now = Local::now();

    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name_in_dir(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir)?;
    apply_migrations(&temp_dir, &db_name)?;

    let config_file_path = temp_dir.join(".surrealdb");

    let configuration = SurrealdbConfiguration {
        db: Some(db_name),
        ..Default::default()
    };

    let db = create_surrealdb_client(&configuration).await?;

    let runner = MigrationRunner::new(&db).use_config_file(&config_file_path);

    let migrations_applied = runner.list().await?;

    ensure!(
        migrations_applied.len() == 3,
        "Expected 3 migrations to be applied"
    );

    let date_prefix = now.format("%Y%m%d_%H%M").to_string();

    let now_timestamp = now.timestamp();
    let now_timestamp_range = (now_timestamp - 2)..(now_timestamp + 2);

    let first_migration = migrations_applied.first()
        .context("Cannot get first migration")?;

    ensure!(
        first_migration.script_name == format!("{}01_AddAdminUser", date_prefix),
        "Expected first migration script name to be {}01_AddAdminUser",
        date_prefix
    );
    ensure!(
        now_timestamp_range.contains(
            &DateTime::parse_from_rfc3339(&first_migration.executed_at)
                .map(|dt| dt.timestamp())
                .context("Cannot parse first migration execution date")?
        ),
        "Expected first migration to be executed just now"
    );

    let second_migration = migrations_applied
        .get(1)
        .context("Cannot get second migration")?;

    ensure!(
        second_migration.script_name == format!("{}02_AddPost", date_prefix),
        "Expected second migration script name to be {}02_AddPost",
        date_prefix
    );
    ensure!(
        now_timestamp_range.contains(
            &DateTime::parse_from_rfc3339(&second_migration.executed_at)
                .map(|dt| dt.timestamp())
                .context("Cannot parse second migration execution date")?
        ),
        "Expected second migration to be executed just now"
    );

    let third_migration = migrations_applied
        .get(2)
        .context("Cannot get third migration")?;

    ensure!(
        third_migration.script_name == format!("{}03_CommentPost", date_prefix),
        "Expected third migration script name to be {}03_CommentPost",
        date_prefix
    );
    ensure!(
        now_timestamp_range.contains(
            &DateTime::parse_from_rfc3339(&third_migration.executed_at)
                .map(|dt| dt.timestamp())
                .context("Cannot parse third migration execution date")?
        ),
        "Expected third migration to be executed just now"
    );

    Ok(())
}
