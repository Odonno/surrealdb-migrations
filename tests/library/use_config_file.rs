use color_eyre::eyre::{ensure, Result};
use surrealdb_migrations::MigrationRunner;

use crate::helpers::*;

#[tokio::test]
async fn use_config_file_fails_if_folder_does_not_exist() -> Result<()> {
    let configuration = SurrealdbConfiguration::default();
    let db = create_surrealdb_client(&configuration).await?;

    let result = MigrationRunner::new(&db)
        .use_config_file(&"/temp/void/.surrealdb")
        .up()
        .await;

    ensure!(
        result.as_ref().is_err(),
        "Expected error, but got {:?}",
        result
    );
    ensure!(
        result.as_ref().err().unwrap().to_string() == "The file 'script_migration' should exist.",
        "Expected error message to be 'The file 'script_migration' should exist.', but got {:?}",
        result.as_ref().err().unwrap().to_string()
    );

    Ok(())
}
