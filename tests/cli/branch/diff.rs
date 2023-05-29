use anyhow::Result;
use serial_test::serial;

use crate::helpers::*;

#[tokio::test]
#[serial]
async fn diff_without_changes() -> Result<()> {
    run_with_surreal_instance_async(|| {
        Box::pin(async {
            clear_tests_files()?;
            scaffold_blog_template()?;
            apply_migrations()?;
            create_branch("test-branch")?;

            let mut cmd = create_cmd()?;

            cmd.arg("branch").arg("diff").arg("test-branch");

            cmd.assert()
                .try_success()
                .and_then(|assert| assert.try_stdout("No schema changes detected\n"))?;

            Ok(())
        })
    })
    .await
}

#[tokio::test]
#[serial]
async fn diff_with_changes() -> Result<()> {
    run_with_surreal_instance_async(|| {
        Box::pin(async {
            clear_tests_files()?;
            scaffold_blog_template()?;
            apply_migrations()?;
            create_branch("test-branch")?;
            add_new_schema_file()?;
            apply_migrations_on_branch("test-branch")?;

            let mut cmd = create_cmd()?;

            cmd.arg("branch").arg("diff").arg("test-branch");

            cmd.assert().try_success().and_then(|assert| {
                assert.try_stdout(
                    "Schema changes detected:

### 1 tables created ###

## category ##

DEFINE TABLE category SCHEMALESS
DEFINE FIELD created_at ON category TYPE datetime VALUE $before OR time::now()
DEFINE FIELD name ON category TYPE string\n",
                )
            })?;

            Ok(())
        })
    })
    .await
}

#[tokio::test]
#[serial]
async fn fails_if_branch_does_not_exist() -> Result<()> {
    run_with_surreal_instance_async(|| {
        Box::pin(async {
            clear_tests_files()?;

            let mut cmd = create_cmd()?;

            cmd.arg("branch").arg("diff").arg("void");

            cmd.assert()
                .try_failure()
                .and_then(|assert| assert.try_stderr("Error: Branch void does not exist\n"))?;

            Ok(())
        })
    })
    .await
}
