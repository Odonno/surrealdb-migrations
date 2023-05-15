use anyhow::{ensure, Result};
use serial_test::serial;
use surrealdb_migrations::{SurrealdbConfiguration, SurrealdbMigrations};

use crate::helpers::*;

#[tokio::test]
#[serial]
async fn apply_with_skipped_migrations() -> Result<()> {
    run_with_surreal_instance_async(|| {
        Box::pin(async {
            clear_tests_files()?;
            scaffold_blog_template()?;

            let first_migration_name = get_first_migration_name()?;

            let configuration = SurrealdbConfiguration::default();
            let runner = SurrealdbMigrations::new(configuration);

            runner.up_to(&first_migration_name).await?;

            let migrations_applied = runner.list().await?;
            ensure!(migrations_applied.len() == 1);

            Ok(())
        })
    })
    .await
}
