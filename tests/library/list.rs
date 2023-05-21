use anyhow::{ensure, Context, Result};
use chrono::{DateTime, Local};
use serial_test::serial;
use surrealdb_migrations::SurrealdbMigrations;

use crate::helpers::*;

#[tokio::test]
#[serial]
async fn list_empty_migrations() -> Result<()> {
    run_with_surreal_instance_async(|| {
        Box::pin(async {
            clear_tests_files()?;
            scaffold_empty_template()?;
            apply_migrations()?;

            let configuration = SurrealdbConfiguration::default();
            let db = create_surrealdb_client(&configuration).await?;

            let migrations_applied = SurrealdbMigrations::new(&db).list().await?;

            ensure!(migrations_applied.len() == 0);

            Ok(())
        })
    })
    .await
}

#[tokio::test]
#[serial]
async fn list_blog_migrations() -> Result<()> {
    run_with_surreal_instance_async(|| {
        Box::pin(async {
            let now = Local::now();

            clear_tests_files()?;
            scaffold_blog_template()?;
            apply_migrations()?;

            let configuration = SurrealdbConfiguration::default();
            let db = create_surrealdb_client(&configuration).await?;

            let migrations_applied = SurrealdbMigrations::new(&db).list().await?;

            ensure!(migrations_applied.len() == 3);

            let date_prefix = now.format("%Y%m%d_%H%M").to_string();

            let now_timestamp = now.timestamp();
            let now_timestamp_range = (now_timestamp - 2)..(now_timestamp + 2);

            let first_migration = migrations_applied
                .get(0)
                .context("Cannot get first migration")?;

            ensure!(first_migration.script_name == format!("{}01_AddAdminUser", date_prefix));
            ensure!(now_timestamp_range.contains(
                &DateTime::parse_from_rfc3339(&first_migration.executed_at)
                    .map(|dt| dt.timestamp())
                    .context("Cannot parse first migration execution date")?
            ));

            let second_migration = migrations_applied
                .get(1)
                .context("Cannot get second migration")?;

            ensure!(second_migration.script_name == format!("{}02_AddPost", date_prefix));
            ensure!(now_timestamp_range.contains(
                &DateTime::parse_from_rfc3339(&second_migration.executed_at)
                    .map(|dt| dt.timestamp())
                    .context("Cannot parse second migration execution date")?
            ));

            let third_migration = migrations_applied
                .get(2)
                .context("Cannot get third migration")?;

            ensure!(third_migration.script_name == format!("{}03_CommentPost", date_prefix));
            ensure!(now_timestamp_range.contains(
                &DateTime::parse_from_rfc3339(&third_migration.executed_at)
                    .map(|dt| dt.timestamp())
                    .context("Cannot parse third migration execution date")?
            ));

            Ok(())
        })
    })
    .await
}
