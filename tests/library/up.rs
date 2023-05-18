use anyhow::Result;
use serial_test::serial;
use surrealdb_migrations::SurrealdbMigrations;

use crate::helpers::*;

#[tokio::test]
#[serial]
async fn apply_initial_schema_changes() -> Result<()> {
    run_with_surreal_instance_async(|| {
        Box::pin(async {
            clear_tests_files()?;
            scaffold_blog_template()?;
            remove_folder("tests-files/migrations")?;

            let configuration = SurrealdbConfiguration::default();
            let db = create_surrealdb_client(&configuration).await?;

            SurrealdbMigrations::new(db).up().await?;

            Ok(())
        })
    })
    .await
}

#[tokio::test]
#[serial]
async fn apply_new_schema_changes() -> Result<()> {
    run_with_surreal_instance_async(|| {
        Box::pin(async {
            clear_tests_files()?;
            scaffold_blog_template()?;
            empty_folder("tests-files/migrations")?;
            apply_migrations()?;
            add_new_schema_file()?;

            let configuration = SurrealdbConfiguration::default();
            let db = create_surrealdb_client(&configuration).await?;

            SurrealdbMigrations::new(db).up().await?;

            Ok(())
        })
    })
    .await
}

#[tokio::test]
#[serial]
async fn apply_initial_migrations() -> Result<()> {
    run_with_surreal_instance_async(|| {
        Box::pin(async {
            clear_tests_files()?;
            scaffold_blog_template()?;

            let configuration = SurrealdbConfiguration::default();
            let db = create_surrealdb_client(&configuration).await?;

            SurrealdbMigrations::new(db).up().await?;

            Ok(())
        })
    })
    .await
}

#[tokio::test]
#[serial]
async fn apply_new_migrations() -> Result<()> {
    run_with_surreal_instance_async(|| {
        Box::pin(async {
            clear_tests_files()?;
            scaffold_blog_template()?;

            let first_migration_name = get_first_migration_name()?;
            apply_migrations_up_to(&first_migration_name)?;

            let configuration = SurrealdbConfiguration::default();
            let db = create_surrealdb_client(&configuration).await?;

            SurrealdbMigrations::new(db).up().await?;

            Ok(())
        })
    })
    .await
}

#[tokio::test]
#[serial]
async fn apply_with_db_configuration() -> Result<()> {
    run_with_surreal_instance_with_admin_user_async(|| {
        Box::pin(async {
            clear_tests_files()?;
            scaffold_blog_template()?;
            empty_folder("tests-files/migrations")?;

            let configuration = SurrealdbConfiguration {
                address: None,
                url: None,
                username: Some("admin".to_string()),
                password: Some("admin".to_string()),
                ns: Some("namespace".to_string()),
                db: Some("database".to_string()),
            };
            let db = create_surrealdb_client(&configuration).await?;

            SurrealdbMigrations::new(db).up().await?;

            Ok(())
        })
    })
    .await
}

#[tokio::test]
#[serial]
async fn apply_should_skip_events_if_no_events_folder() -> Result<()> {
    run_with_surreal_instance_with_admin_user_async(|| {
        Box::pin(async {
            clear_tests_files()?;
            scaffold_blog_template()?;
            empty_folder("tests-files/migrations")?;
            remove_folder("tests-files/events")?;

            let configuration = SurrealdbConfiguration {
                address: None,
                url: None,
                username: Some("admin".to_string()),
                password: Some("admin".to_string()),
                ns: Some("namespace".to_string()),
                db: Some("database".to_string()),
            };
            let db = create_surrealdb_client(&configuration).await?;

            SurrealdbMigrations::new(db).up().await?;

            Ok(())
        })
    })
    .await
}
