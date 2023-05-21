use anyhow::{ensure, Result};
use regex::Regex;
use serial_test::serial;
use surrealdb_migrations::SurrealdbMigrations;

use crate::helpers::*;

#[tokio::test]
#[serial]
async fn ok_if_no_migration_file() -> Result<()> {
    run_with_surreal_instance_async(|| {
        Box::pin(async {
            clear_tests_files()?;
            scaffold_empty_template()?;

            let configuration = SurrealdbConfiguration::default();
            let db = create_surrealdb_client(&configuration).await?;

            let runner = SurrealdbMigrations::new(&db);

            runner.validate_version_order().await?;

            Ok(())
        })
    })
    .await
}

#[tokio::test]
#[serial]
async fn ok_if_migrations_applied_but_no_new_migration() -> Result<()> {
    run_with_surreal_instance_async(|| {
        Box::pin(async {
            clear_tests_files()?;
            scaffold_blog_template()?;

            let configuration = SurrealdbConfiguration::default();
            let db = create_surrealdb_client(&configuration).await?;

            let runner = SurrealdbMigrations::new(&db);

            runner.up().await?;

            runner.validate_version_order().await?;

            Ok(())
        })
    })
    .await
}

#[tokio::test]
#[serial]
async fn ok_if_migrations_applied_with_new_migration_after_last_applied() -> Result<()> {
    run_with_surreal_instance_async(|| {
        Box::pin(async {
            clear_tests_files()?;
            scaffold_blog_template()?;

            let configuration = SurrealdbConfiguration::default();
            let db = create_surrealdb_client(&configuration).await?;

            let runner = SurrealdbMigrations::new(&db);

            let first_migration_name = get_first_migration_name()?;
            runner.up_to(&first_migration_name).await?;

            runner.validate_version_order().await?;

            Ok(())
        })
    })
    .await
}

#[tokio::test]
#[serial]
async fn fails_if_migrations_applied_with_new_migration_before_last_applied() -> Result<()> {
    run_with_surreal_instance_async(|| {
        Box::pin(async {
            clear_tests_files()?;
            scaffold_blog_template()?;

            let configuration = SurrealdbConfiguration::default();
            let db = create_surrealdb_client(&configuration).await?;

            let runner = SurrealdbMigrations::new(&db);

            let first_migration_file = get_first_migration_file()?;
            std::fs::remove_file(first_migration_file)?;

            runner.up().await?;

            clear_tests_files()?;
            scaffold_blog_template()?;

            let result = runner.validate_version_order().await;

            ensure!(result.is_err());

            let error_regex = Regex::new(
                r"The following migrations have not been applied: \d+_\d+_AddAdminUser",
            )?;

            let error_str = result.unwrap_err().to_string();
            let error_str = error_str.as_str();

            ensure!(error_regex.is_match(error_str));

            Ok(())
        })
    })
    .await
}
