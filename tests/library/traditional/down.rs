use assert_fs::TempDir;
use color_eyre::{eyre::ensure, Result};
use surrealdb_migrations::MigrationRunner;

use crate::helpers::*;

#[tokio::test]
async fn apply_revert_all_migrations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name_in_dir(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, true)?;

    let config_file_path = temp_dir.join(".surrealdb");

    let configuration = SurrealdbConfiguration {
        db: Some(db_name.to_string()),
        ..Default::default()
    };

    let db = create_surrealdb_client(&configuration).await?;

    let runner = MigrationRunner::new(&db).use_config_file(&config_file_path);

    runner.up().await?;

    runner.down().await?;

    let migrations_applied = runner.list().await?;
    ensure!(
        migrations_applied.is_empty(),
        "Expected no migrations to be applied"
    );

    temp_dir.close()?;

    Ok(())
}
