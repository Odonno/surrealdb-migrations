use anyhow::{anyhow, ensure, Result};
use regex::Regex;
use serial_test::serial;

use crate::helpers::*;

#[tokio::test]
#[serial]
async fn create_new_branch() -> Result<()> {
    run_with_surreal_instance_async(|| {
        Box::pin(async {
            clear_tests_files()?;
            scaffold_blog_template()?;
            apply_migrations()?;

            let mut cmd = create_cmd()?;

            cmd.arg("branch")
                .arg("new")
                .arg("test-branch")
                .arg("--address")
                .arg("http://localhost:8000");

            cmd.assert().try_success().and_then(|assert| {
                assert.try_stdout(
                    "You can now use the branch with the following configuration:

ns: branches
db: test-branch\n",
                )
            })?;

            // Check "branch" record exist in surrealdb
            let branch: Option<Branch> = get_surrealdb_record(
                "database".to_string(),
                "branching".to_string(),
                "branch".to_string(),
                "test-branch".to_string(),
            )
            .await?;

            ensure!(branch.is_some(), "Branch record should exist");

            // Check database is replicated in surrealdb
            let is_empty = is_surrealdb_empty(
                Some("branches".to_string()),
                Some("test-branch".to_string()),
            )
            .await?;

            ensure!(!is_empty, "SurrealDB should not be empty");

            Ok(())
        })
    })
    .await
}

#[tokio::test]
#[serial]
async fn fails_if_branch_already_exists() -> Result<()> {
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

                cmd.arg("branch")
                    .arg("new")
                    .arg("test-branch")
                    .arg("--address")
                    .arg("http://localhost:8000");

                cmd.assert()
                    .try_failure()
                    .and_then(|assert| assert.try_stderr("Error: Branch name already exists\n"))?;
            }

            Ok(())
        })
    })
    .await
}

#[tokio::test]
#[serial]
async fn create_new_branch_with_random_name() -> Result<()> {
    run_with_surreal_instance_async(|| {
        Box::pin(async {
            clear_tests_files()?;
            scaffold_blog_template()?;
            apply_migrations()?;

            let mut cmd = create_cmd()?;

            cmd.arg("branch")
                .arg("new")
                .arg("--address")
                .arg("http://localhost:8000");

            let output = cmd.assert().try_success()?.get_output().stdout.to_owned();
            let output = String::from_utf8(output)?;

            println!("output: {:?}", output);

            let regex = Regex::new(
                r"^You can now use the branch with the following configuration:

ns: branches
db: (\S+)\n$",
            )?;

            let branch_name = regex
                .captures(&output)
                .ok_or_else(|| anyhow!("Output should match regex #1"))?
                .get(1)
                .ok_or_else(|| anyhow!("Output should match regex #2"))?
                .as_str();

            // Check "branch" record exist in surrealdb
            let branch: Option<Branch> = get_surrealdb_record(
                "database".to_string(),
                "branching".to_string(),
                "branch".to_string(),
                branch_name.to_string(),
            )
            .await?;

            ensure!(branch.is_some(), "Branch record should exist");

            // Check database is replicated in surrealdb
            let is_empty =
                is_surrealdb_empty(Some("branches".to_string()), Some(branch_name.to_string()))
                    .await?;

            ensure!(!is_empty, "SurrealDB should not be empty");

            Ok(())
        })
    })
    .await
}
