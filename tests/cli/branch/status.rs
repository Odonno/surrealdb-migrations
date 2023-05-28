use anyhow::{ensure, Result};
use regex::Regex;
use serial_test::serial;

use crate::helpers::*;

#[tokio::test]
#[serial]
async fn display_branch_status() -> Result<()> {
    run_with_surreal_instance_async(|| {
        Box::pin(async {
            clear_tests_files()?;
            scaffold_blog_template()?;
            apply_migrations()?;
            create_branch("test-branch")?;

            let mut cmd = create_cmd()?;

            cmd.arg("branch").arg("status").arg("test-branch");

            let output = cmd.assert().try_success()?.get_output().stdout.to_owned();
            let output = String::from_utf8(output)?;

            let regex = Regex::new(
                r"## Branch status ##
Name: test-branch
Namespace: branches
Database: test-branch
Created at: (.+) \(just now\)

## Origin Branch ##
Namespace: test
Database: test\n",
            )?;

            let match_regex = regex.is_match(&output);

            ensure!(match_regex, "Output does not match regex");

            Ok(())
        })
    })
    .await
}

#[tokio::test]
#[serial]
async fn display_branch_status_using_alias() -> Result<()> {
    run_with_surreal_instance_async(|| {
        Box::pin(async {
            clear_tests_files()?;
            scaffold_blog_template()?;
            apply_migrations()?;
            create_branch("test-branch")?;

            let mut cmd = create_cmd()?;

            cmd.arg("branch").arg("test-branch");

            let output = cmd.assert().try_success()?.get_output().stdout.to_owned();
            let output = String::from_utf8(output)?;

            let regex = Regex::new(
                r"## Branch status ##
Name: test-branch
Namespace: branches
Database: test-branch
Created at: (.+) \(just now\)

## Origin Branch ##
Namespace: test
Database: test\n",
            )?;

            let match_regex = regex.is_match(&output);

            ensure!(match_regex, "Output does not match regex");

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

            cmd.arg("branch").arg("status").arg("void");

            cmd.assert()
                .try_failure()
                .and_then(|assert| assert.try_stderr("Error: Branch void does not exist\n"))?;

            Ok(())
        })
    })
    .await
}
