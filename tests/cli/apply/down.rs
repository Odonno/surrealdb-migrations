use anyhow::Result;
use serial_test::serial;

use crate::helpers::*;

#[tokio::test]
#[serial]
async fn apply_revert_all_migrations() -> Result<()> {
    run_with_surreal_instance(|| {
        clear_tests_files()?;
        scaffold_blog_template()?;

        {
            let mut cmd = create_cmd()?;

            cmd.arg("apply");

            cmd.assert().try_success()?;
        }

        {
            let mut cmd = create_cmd()?;

            cmd.arg("apply").arg("--down").arg("0");

            cmd.assert().try_success().and_then(|assert| {
                assert.try_stdout(
                    "Schema files successfully executed!
Event files successfully executed!
Reverting migration CommentPost...
Reverting migration AddPost...
Reverting migration AddAdminUser...
Migration files successfully executed!\n",
                )
            })?;
        }

        Ok(())
    })
}

#[tokio::test]
#[serial]
async fn apply_revert_to_first_migration() -> Result<()> {
    run_with_surreal_instance(|| {
        clear_tests_files()?;
        scaffold_blog_template()?;

        let first_migration_name = get_first_migration_name()?;

        {
            let mut cmd = create_cmd()?;

            cmd.arg("apply");

            cmd.assert().try_success()?;
        }

        {
            let mut cmd = create_cmd()?;

            cmd.arg("apply").arg("--down").arg(first_migration_name);

            cmd.assert().try_success().and_then(|assert| {
                assert.try_stdout(
                    "Schema files successfully executed!
Event files successfully executed!
Reverting migration CommentPost...
Reverting migration AddPost...
Migration files successfully executed!\n",
                )
            })?;
        }

        Ok(())
    })
}
