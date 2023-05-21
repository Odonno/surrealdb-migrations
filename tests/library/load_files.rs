use anyhow::Result;
use include_dir::{include_dir, Dir};
use serial_test::serial;
use surrealdb_migrations::MigrationRunner;

use crate::helpers::*;

#[tokio::test]
#[serial]
async fn load_files_from_empty_template() -> Result<()> {
    run_with_surreal_instance_async(|| {
        Box::pin(async {
            clear_tests_files()?;

            let configuration = SurrealdbConfiguration::default();
            let db = create_surrealdb_client(&configuration).await?;

            const EMPTY_TEMPLATE: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/empty");

            MigrationRunner::new(&db).load_files(&EMPTY_TEMPLATE);

            Ok(())
        })
    })
    .await
}

#[tokio::test]
#[serial]
async fn load_files_from_blog_template() -> Result<()> {
    run_with_surreal_instance_async(|| {
        Box::pin(async {
            clear_tests_files()?;

            let configuration = SurrealdbConfiguration::default();
            let db = create_surrealdb_client(&configuration).await?;

            const BLOG_TEMPLATE: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/blog");

            MigrationRunner::new(&db).load_files(&BLOG_TEMPLATE);

            Ok(())
        })
    })
    .await
}

#[tokio::test]
#[serial]
async fn load_files_from_ecommerce_template() -> Result<()> {
    run_with_surreal_instance_async(|| {
        Box::pin(async {
            clear_tests_files()?;

            let configuration = SurrealdbConfiguration::default();
            let db = create_surrealdb_client(&configuration).await?;

            const ECOMMERCE_TEMPLATE: Dir<'_> =
                include_dir!("$CARGO_MANIFEST_DIR/templates/ecommerce");

            MigrationRunner::new(&db).load_files(&ECOMMERCE_TEMPLATE);

            Ok(())
        })
    })
    .await
}

#[tokio::test]
#[serial]
async fn validate_version_order_from_embedded_files() -> Result<()> {
    run_with_surreal_instance_async(|| {
        Box::pin(async {
            clear_tests_files()?;

            const BLOG_TEMPLATE: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/blog");

            let configuration = SurrealdbConfiguration::default();
            let db = create_surrealdb_client(&configuration).await?;

            MigrationRunner::new(&db)
                .load_files(&BLOG_TEMPLATE)
                .validate_version_order()
                .await?;

            Ok(())
        })
    })
    .await
}

#[tokio::test]
#[serial]
async fn apply_migrations_from_embedded_files() -> Result<()> {
    run_with_surreal_instance_async(|| {
        Box::pin(async {
            clear_tests_files()?;

            const BLOG_TEMPLATE: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/blog");

            let configuration = SurrealdbConfiguration::default();
            let db = create_surrealdb_client(&configuration).await?;

            MigrationRunner::new(&db)
                .load_files(&BLOG_TEMPLATE)
                .up()
                .await?;

            Ok(())
        })
    })
    .await
}
