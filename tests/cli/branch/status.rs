use anyhow::Result;
use assert_fs::TempDir;
use regex::Regex;
use serial_test::serial;

use crate::helpers::*;

#[tokio::test]
#[serial("branches")]
async fn display_branch_status() -> Result<()> {
    remove_features_ns().await?;

    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, &db_name)?;
    scaffold_blog_template(&temp_dir)?;
    apply_migrations(&temp_dir, &db_name)?;
    create_branch(&temp_dir, "test-branch")?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("branch").arg("status").arg("test-branch");

    let output = cmd.assert().try_success()?.get_output().stdout.to_owned();
    let output = String::from_utf8(output)?;

    let regex = Regex::new(&format!(
        r"## Branch status ##
Name: test-branch
Namespace: branches
Database: test-branch
Created at: (.+) \(just now\)

## Origin Branch ##
Namespace: test
Database: {db_name}\n",
    ))?;

    let match_regex = regex.is_match(&output);
    assert!(match_regex, "Output does not match regex: {output}");

    Ok(())
}

#[tokio::test]
#[serial("branches")]
async fn display_branch_status_using_alias() -> Result<()> {
    remove_features_ns().await?;

    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, &db_name)?;
    scaffold_blog_template(&temp_dir)?;
    apply_migrations(&temp_dir, &db_name)?;
    create_branch(&temp_dir, "test-branch")?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("branch").arg("test-branch");

    let output = cmd.assert().try_success()?.get_output().stdout.to_owned();
    let output = String::from_utf8(output)?;

    let regex = Regex::new(&format!(
        r"## Branch status ##
Name: test-branch
Namespace: branches
Database: test-branch
Created at: (.+) \(just now\)

## Origin Branch ##
Namespace: test
Database: {db_name}\n",
    ))?;

    let match_regex = regex.is_match(&output);
    assert!(match_regex, "Output does not match regex: {output}");

    Ok(())
}

#[tokio::test]
#[serial("branches")]
async fn fails_if_branch_does_not_exist() -> Result<()> {
    remove_features_ns().await?;

    let temp_dir = TempDir::new()?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("branch").arg("status").arg("void");

    cmd.assert()
        .try_failure()
        .and_then(|assert| assert.try_stderr("Error: Branch void does not exist\n"))?;

    Ok(())
}
