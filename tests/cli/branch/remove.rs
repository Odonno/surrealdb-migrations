use anyhow::{ensure, Result};
use serial_test::serial;

use crate::helpers::*;

#[tokio::test]
#[serial]
async fn remove_existing_branch() -> Result<()> {
    run_with_surreal_instance_async(|| {
        Box::pin(async {
            clear_tests_files()?;
            scaffold_blog_template()?;
            apply_migrations()?;

            {
                let mut cmd = create_cmd()?;

                cmd.arg("branch")
                    .arg("new")
                    .arg("test-branch")
                    .arg("--address")
                    .arg("http://localhost:8000");

                cmd.assert().try_success()?;
            }

            {
                let mut cmd = create_cmd()?;

                cmd.arg("branch").arg("remove").arg("test-branch");

                cmd.assert().try_success().and_then(|assert| {
                    assert.try_stdout("Branch test-branch successfully removed\n")
                })?;
            }

            // Check "branch" record does not exist in surrealdb
            let branch: Option<Branch> = get_surrealdb_record(
                "database".to_string(),
                "branching".to_string(),
                "branch".to_string(),
                "test-branch".to_string(),
            )
            .await?;

            ensure!(branch.is_none(), "Branch record should not exist");

            // Check database is removed from surrealdb
            let is_empty = is_surrealdb_empty(
                Some("branches".to_string()),
                Some("test-branch".to_string()),
            )
            .await?;

            ensure!(is_empty, "SurrealDB database should be empty");

            Ok(())
        })
    })
    .await
}

#[tokio::test]
#[serial]
async fn fails_to_remove_if_branch_does_not_exist() -> Result<()> {
    run_with_surreal_instance_async(|| {
        Box::pin(async {
            clear_tests_files()?;

            let mut cmd = create_cmd()?;

            cmd.arg("branch").arg("remove").arg("void");

            cmd.assert()
                .try_failure()
                .and_then(|assert| assert.try_stderr("Error: Branch void does not exist\n"))?;

            Ok(())
        })
    })
    .await
}
