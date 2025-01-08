use assert_fs::TempDir;
use color_eyre::{
    eyre::{ensure, eyre},
    Result,
};
use predicates::prelude::*;
use regex::Regex;
use serial_test::serial;

use crate::helpers::*;

#[ignore = "potential issue with create_branch"]
#[tokio::test]
#[serial(branches)]
async fn create_new_branch() -> Result<()> {
    remove_features_ns().await?;

    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir)?;
    apply_migrations(&temp_dir, &db_name)?;

    let mut cmd = create_cmd(&temp_dir)?;

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
        "features".to_string(),
        "branching".to_string(),
        "branch".to_string(),
        "test-branch".to_string(),
    )
    .await?;

    ensure!(branch.is_some(), "Branch record should exist");
    ensure!(
        branch.clone().unwrap().from_ns == "test",
        "Origin branch ns should be test"
    );
    assert_eq!(branch.clone().unwrap().from_db, db_name);

    // Check database is replicated in surrealdb
    let is_empty = is_surreal_db_empty(
        Some("branches".to_string()),
        Some("test-branch".to_string()),
    )
    .await?;

    ensure!(!is_empty, "SurrealDB database should not be empty");

    // Check origin database is replicated in surrealdb
    let is_empty = is_surreal_db_empty(
        Some("branches/origin".to_string()),
        Some("test-branch".to_string()),
    )
    .await?;

    ensure!(!is_empty, "SurrealDB database should not be empty");

    Ok(())
}

#[ignore = "potential issue with create_branch"]
#[tokio::test]
#[serial(branches)]
async fn fails_if_branch_already_exists() -> Result<()> {
    remove_features_ns().await?;

    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir)?;
    apply_migrations(&temp_dir, &db_name)?;
    create_branch(&temp_dir, "test-branch")?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("branch")
        .arg("new")
        .arg("test-branch")
        .arg("--address")
        .arg("http://localhost:8000");

    cmd.assert().try_failure().and_then(|assert| {
        assert.try_stderr(predicate::str::contains("Branch name already exists"))
    })?;

    Ok(())
}

#[ignore = "potential issue with create_branch"]
#[tokio::test]
#[serial(branches)]
async fn create_new_branch_with_random_name() -> Result<()> {
    remove_features_ns().await?;

    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir)?;
    apply_migrations(&temp_dir, &db_name)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("branch")
        .arg("new")
        .arg("--address")
        .arg("http://localhost:8000");

    let output = cmd.assert().try_success()?.get_output().stdout.to_owned();
    let output = String::from_utf8(output)?;

    let regex = Regex::new(
        r"^You can now use the branch with the following configuration:

ns: branches
db: (\S+)\n$",
    )?;

    let branch_name = regex
        .captures(&output)
        .ok_or_else(|| eyre!("Output should match regex #1"))?
        .get(1)
        .ok_or_else(|| eyre!("Output should match regex #2"))?
        .as_str();

    // Check "branch" record exist in surrealdb
    let branch: Option<Branch> = get_surrealdb_record(
        "features".to_string(),
        "branching".to_string(),
        "branch".to_string(),
        branch_name.to_string(),
    )
    .await?;

    ensure!(branch.is_some(), "Branch record should exist");
    ensure!(
        branch.clone().unwrap().from_ns == "test",
        "Origin branch ns should be test"
    );
    assert_eq!(branch.clone().unwrap().from_db, db_name);

    // Check database is replicated in surrealdb
    let is_empty =
        is_surreal_db_empty(Some("branches".to_string()), Some(branch_name.to_string())).await?;

    ensure!(!is_empty, "SurrealDB database should not be empty");

    // Check origin database is replicated in surrealdb
    let is_empty = is_surreal_db_empty(
        Some("branches/origin".to_string()),
        Some(branch_name.to_string()),
    )
    .await?;

    ensure!(!is_empty, "SurrealDB database should not be empty");

    Ok(())
}

#[ignore = "potential issue with create_branch"]
#[tokio::test]
#[serial(branches)]
async fn create_new_branch_using_config_file() -> Result<()> {
    remove_features_ns().await?;

    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir)?;
    apply_migrations(&temp_dir, &db_name)?;

    let mut cmd = create_cmd(&temp_dir)?;

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
        "features".to_string(),
        "branching".to_string(),
        "branch".to_string(),
        "test-branch".to_string(),
    )
    .await?;

    ensure!(branch.is_some(), "Branch record should exist");
    ensure!(
        branch.clone().unwrap().from_ns == "test",
        "Origin branch ns should be test"
    );
    ensure!(
        branch.clone().unwrap().from_db == db_name,
        format!("Origin branch db should be {}", db_name)
    );

    // Check database is replicated in surrealdb
    let is_empty = is_surreal_db_empty(
        Some("branches".to_string()),
        Some("test-branch".to_string()),
    )
    .await?;

    ensure!(!is_empty, "SurrealDB database should not be empty");

    // Check origin database is replicated in surrealdb
    let is_empty = is_surreal_db_empty(
        Some("branches/origin".to_string()),
        Some("test-branch".to_string()),
    )
    .await?;

    ensure!(!is_empty, "SurrealDB database should not be empty");

    Ok(())
}
