use assert_fs::TempDir;
use color_eyre::eyre::{Error, Result};
use insta::{assert_ron_snapshot, assert_snapshot, Settings};
use itertools::Itertools;

use crate::helpers::*;

#[tokio::test]
async fn apply_revert_single_migration() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;
    apply_migrations(&temp_dir, &db_name)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply").arg("--down");

    let assert = cmd.assert().try_success()?;
    let stdout = get_stdout_str(assert)?;

    let mut insta_settings = Settings::new();
    insta_settings.bind(|| {
        assert_snapshot!(stdout, @r"
        Reverting migration CommentPost...
        Migration files successfully executed!
        ");
        Ok::<(), Error>(())
    })?;

    let script_migrations: Vec<ScriptMigration> = get_surrealdb_records(
        "test".to_string(),
        db_name.to_string(),
        "script_migration".to_string(),
    )
    .await?;

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
