use anyhow::Result;
use serial_test::serial;
use surrealdb_migrations::{SurrealdbConfiguration, SurrealdbMigrations};

use crate::helpers::common::*;

#[tokio::test]
#[serial]
async fn apply_initial_schema_changes() -> Result<()> {
    run_with_surreal_instance_async(|| {
        Box::pin(async {
            clear_files_dir()?;
            scaffold_blog_template()?;
            empty_folder("tests-files/migrations")?;

            let configuration = SurrealdbConfiguration::default();
            SurrealdbMigrations::new(configuration).up().await?;

            Ok(())
        })
    })
    .await
}

#[tokio::test]
#[serial]
async fn cannot_apply_if_surreal_instance_not_running() -> Result<()> {
    clear_files_dir()?;
    scaffold_blog_template()?;

    let configuration = SurrealdbConfiguration::default();
    let result = SurrealdbMigrations::new(configuration).up().await;

    let error = result.unwrap_err();

    assert_eq!(
        error.to_string(),
        "There was an error processing a remote WS request"
    );

    Ok(())
}

#[tokio::test]
#[serial]
async fn apply_new_schema_changes() -> Result<()> {
    run_with_surreal_instance_async(|| {
        Box::pin(async {
            clear_files_dir()?;
            scaffold_blog_template()?;
            empty_folder("tests-files/migrations")?;
            apply_migrations()?;
            add_new_schema_file()?;

            let configuration = SurrealdbConfiguration::default();
            SurrealdbMigrations::new(configuration).up().await?;

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
            clear_files_dir()?;
            scaffold_blog_template()?;

            let configuration = SurrealdbConfiguration::default();
            SurrealdbMigrations::new(configuration).up().await?;

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
            clear_files_dir()?;
            scaffold_blog_template()?;

            let first_migration_name = get_first_migration_name()?;
            apply_migrations_up_to(&first_migration_name)?;

            let configuration = SurrealdbConfiguration::default();
            SurrealdbMigrations::new(configuration).up().await?;

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
            clear_files_dir()?;
            scaffold_blog_template()?;
            empty_folder("tests-files/migrations")?;

            let configuration = SurrealdbConfiguration {
                url: None,
                username: Some("admin".to_string()),
                password: Some("admin".to_string()),
                ns: Some("namespace".to_string()),
                db: Some("database".to_string()),
            };
            SurrealdbMigrations::new(configuration).up().await?;

            Ok(())
        })
    })
    .await
}
