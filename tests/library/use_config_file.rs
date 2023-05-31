use anyhow::{ensure, Result};
use serial_test::serial;
use surrealdb_migrations::MigrationRunner;

use crate::helpers::*;

#[tokio::test]
#[serial]
async fn use_config_file_fails_if_folder_does_not_exist() -> Result<()> {
    run_with_surreal_instance_async(|| {
        Box::pin(async {
            clear_tests_files()?;
            scaffold_blog_template()?;

            let configuration = SurrealdbConfiguration::default();
            let db = create_surrealdb_client(&configuration).await?;

            let result = MigrationRunner::new(&db)
                .use_config_file(".surrealdb-alt")
                .up()
                .await;

            ensure!(
                result.as_ref().is_err(),
                "Expected error, but got {:?}",
                result
            );
            ensure!(
                result.as_ref().err().unwrap().to_string() == "Error listing schemas directory",
                "Expected error message to be 'Error listing schemas directory', but got {:?}",
                result.as_ref().err().unwrap().to_string()
            );

            Ok(())
        })
    })
    .await
}
