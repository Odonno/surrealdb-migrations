use anyhow::{ensure, Result};
use serial_test::serial;

use crate::helpers::*;

#[tokio::test]
#[serial]
async fn merge_existing_branch() -> Result<()> {
    run_with_surreal_instance_async(|| {
        Box::pin(async {
            clear_tests_files()?;
            scaffold_blog_template()?;

            let first_migration_name = get_first_migration_name()?;
            apply_migrations_up_to(&first_migration_name)?;

            create_branch("test-branch")?;

            apply_migrations_on_branch("test-branch")?;

            // Check no data exists in the main branch
            let posts: Vec<Post> =
                get_surrealdb_records("test".to_string(), "test".to_string(), "post".to_string())
                    .await?;

            ensure!(posts.is_empty(), "There should be no post");

            let mut cmd = create_cmd()?;

            cmd.arg("branch")
                .arg("merge")
                .arg("test-branch")
                .arg("--mode")
                .arg("overwrite")
                .arg("--address")
                .arg("http://localhost:8000");

            cmd.assert()
                .try_success()
                .and_then(|assert| assert.try_stdout("Branch test-branch successfully merged\n"))?;

            // Check new data exists in the main branch
            let posts: Vec<Post> =
                get_surrealdb_records("test".to_string(), "test".to_string(), "post".to_string())
                    .await?;

            ensure!(posts.len() == 1, "There should be 1 post");

            // Check "branch" record does not exist in surrealdb
            let branch: Option<Branch> = get_surrealdb_record(
                "features".to_string(),
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
async fn fails_to_merge_if_branch_does_not_exist() -> Result<()> {
    run_with_surreal_instance_async(|| {
        Box::pin(async {
            clear_tests_files()?;

            let mut cmd = create_cmd()?;

            cmd.arg("branch")
                .arg("merge")
                .arg("void")
                .arg("--mode")
                .arg("overwrite");

            cmd.assert()
                .try_failure()
                .and_then(|assert| assert.try_stderr("Error: Branch void does not exist\n"))?;

            Ok(())
        })
    })
    .await
}
