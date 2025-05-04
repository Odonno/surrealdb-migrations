use assert_fs::TempDir;
use color_eyre::eyre::{Error, Result};
use insta::{assert_ron_snapshot, Settings};
use itertools::Itertools;
use surrealdb_migrations::MigrationRunner;

use crate::helpers::*;

#[tokio::test]
async fn set_checksum_on_each_migration_file() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name_in_dir(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;

    let config_file_path = temp_dir.join(".surrealdb");

    let configuration = SurrealdbConfiguration {
        db: Some(db_name),
        ..Default::default()
    };

    let db = create_surrealdb_client(&configuration).await?;

    let runner = MigrationRunner::new(&db).use_config_file(&config_file_path);

    runner.up().await?;

    let script_migrations = runner.list().await?;

    let mut insta_settings = Settings::new();
    insta_settings.add_script_timestamp_filter();
    insta_settings.add_datetime_filter();
    insta_settings.bind(|| {
        assert_ron_snapshot!(script_migrations
            .iter()
            .sorted_by(|a, b| Ord::cmp(&b.script_name, &a.script_name))
            .collect_vec());
        Ok::<(), Error>(())
    })?;

    temp_dir.close()?;

    Ok(())
}
