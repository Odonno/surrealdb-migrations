use color_eyre::Result;
use include_dir::{include_dir, Dir};
use surrealdb_migrations::MigrationRunner;

use crate::helpers::*;

#[tokio::test]
async fn load_files_from_empty_template() -> Result<()> {
    let configuration = SurrealdbConfiguration::default();
    let db = create_surrealdb_client(&configuration).await?;

    const EMPTY_TEMPLATE: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/empty");

    MigrationRunner::new(&db).load_files(&EMPTY_TEMPLATE);

    Ok(())
}

#[tokio::test]
async fn load_files_from_blog_template() -> Result<()> {
    let configuration = SurrealdbConfiguration::default();
    let db = create_surrealdb_client(&configuration).await?;

    const BLOG_TEMPLATE: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/blog");

    MigrationRunner::new(&db).load_files(&BLOG_TEMPLATE);

    Ok(())
}

#[tokio::test]
async fn load_files_from_ecommerce_template() -> Result<()> {
    let configuration = SurrealdbConfiguration::default();
    let db = create_surrealdb_client(&configuration).await?;

    const ECOMMERCE_TEMPLATE: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/ecommerce");

    MigrationRunner::new(&db).load_files(&ECOMMERCE_TEMPLATE);

    Ok(())
}

#[tokio::test]
async fn validate_version_order_from_embedded_files() -> Result<()> {
    const EMBEDDED_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/embedded-files");

    let db_name = generate_random_db_name()?;

    let configuration = SurrealdbConfiguration {
        db: Some(db_name),
        ..Default::default()
    };

    let db = create_surrealdb_client(&configuration).await?;

    MigrationRunner::new(&db)
        .load_files(&EMBEDDED_DIR)
        .validate_version_order()
        .await?;

    Ok(())
}

#[tokio::test]
async fn apply_migrations_from_embedded_files() -> Result<()> {
    const EMBEDDED_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/embedded-files");

    let db_name = generate_random_db_name()?;

    let configuration = SurrealdbConfiguration {
        db: Some(db_name),
        ..Default::default()
    };

    let db = create_surrealdb_client(&configuration).await?;

    MigrationRunner::new(&db)
        .load_files(&EMBEDDED_DIR)
        .up()
        .await?;

    Ok(())
}
