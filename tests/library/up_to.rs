use anyhow::Result;
use serial_test::serial;
use surrealdb_migrations::{SurrealdbConfiguration, SurrealdbMigrations};

use crate::helpers::common::*;

#[tokio::test]
#[serial]
async fn apply_with_skipped_migrations() -> Result<()> {
    run_with_surreal_instance_async(|| {
        Box::pin(async {
            clear_files_dir()?;
            scaffold_blog_template()?;

            let first_migration_name = get_first_migration_name()?;

            let configuration = SurrealdbConfiguration::default();
            SurrealdbMigrations::new(configuration)
                .up_to(&first_migration_name)
                .await?;

            Ok(())
        })
    })
    .await
}
