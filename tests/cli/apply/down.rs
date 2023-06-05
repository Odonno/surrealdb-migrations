use anyhow::{ensure, Result};
use serial_test::serial;

use crate::helpers::*;

#[tokio::test]
#[serial]
async fn apply_revert_all_migrations() -> Result<()> {
    run_with_surreal_instance_async(|| {
        Box::pin(async {
            clear_tests_files()?;
            scaffold_blog_template()?;
            apply_migrations()?;

            let mut cmd = create_cmd()?;

            cmd.arg("apply").arg("--down").arg("0");

            cmd.assert().try_success().and_then(|assert| {
                assert.try_stdout(
                    "Reverting migration CommentPost...
Reverting migration AddPost...
Reverting migration AddAdminUser...
Migration files successfully executed!\n",
                )
            })?;

            let is_table_empty = is_surreal_table_empty(None, "user").await?;
            ensure!(is_table_empty, "'user' table should be empty");

            let is_table_empty = is_surreal_table_empty(None, "post").await?;
            ensure!(is_table_empty, "'post' table should be empty");

            let is_table_empty = is_surreal_table_empty(None, "comment").await?;
            ensure!(is_table_empty, "'comment' table should be empty");

            Ok(())
        })
    })
    .await
}

#[tokio::test]
#[serial]
async fn apply_revert_to_first_migration() -> Result<()> {
    run_with_surreal_instance_async(|| {
        Box::pin(async {
            clear_tests_files()?;
            scaffold_blog_template()?;

            let first_migration_name = get_first_migration_name()?;

            apply_migrations()?;

            let mut cmd = create_cmd()?;

            cmd.arg("apply").arg("--down").arg(first_migration_name);

            cmd.assert().try_success().and_then(|assert| {
                assert.try_stdout(
                    "Reverting migration CommentPost...
Reverting migration AddPost...
Migration files successfully executed!\n",
                )
            })?;

            let is_table_empty = is_surreal_table_empty(None, "user").await?;
            ensure!(!is_table_empty, "'user' table should not be empty");

            let is_table_empty = is_surreal_table_empty(None, "post").await?;
            ensure!(is_table_empty, "'post' table should be empty");

            let is_table_empty = is_surreal_table_empty(None, "comment").await?;
            ensure!(is_table_empty, "'comment' table should be empty");

            Ok(())
        })
    })
    .await
}
