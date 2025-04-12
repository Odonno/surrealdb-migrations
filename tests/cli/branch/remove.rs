use assert_fs::TempDir;
use color_eyre::eyre::{ensure, Result};
use predicates::prelude::*;
use serial_test::serial;

use crate::helpers::*;

#[ignore = "potential issue with create_branch"]
#[tokio::test]
#[serial(branches)]
async fn remove_existing_branch() -> Result<()> {
    remove_features_ns().await?;

    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir)?;
    apply_migrations(&temp_dir, &db_name)?;
    create_branch(&temp_dir, "test-branch")?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("branch").arg("remove").arg("test-branch");

    cmd.assert()
        .try_success()
        .and_then(|assert| assert.try_stdout("Branch test-branch successfully removed\n"))?;

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
    let is_empty = is_surreal_db_empty(
        Some("branches".to_string()),
        Some("test-branch".to_string()),
    )
    .await?;

    ensure!(is_empty, "SurrealDB database should be empty");

    // Check database is removed from surrealdb
    let is_empty = is_surreal_db_empty(
        Some("branches/origin".to_string()),
        Some("test-branch".to_string()),
    )
    .await?;

    ensure!(is_empty, "SurrealDB database should be empty");

    temp_dir.close()?;

    Ok(())
}

#[tokio::test]
#[serial(branches)]
async fn fails_to_remove_if_branch_does_not_exist() -> Result<()> {
    remove_features_ns().await?;

    let temp_dir = TempDir::new()?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("branch").arg("remove").arg("void");

    cmd.assert().try_failure().and_then(|assert| {
        assert.try_stderr(predicate::str::contains("Branch void does not exist"))
    })?;

    temp_dir.close()?;

    Ok(())
}

#[ignore = "potential issue with create_branch"]
#[tokio::test]
#[serial(branches)]
async fn prevent_branch_to_be_removed_if_used_by_another_branch() -> Result<()> {
    remove_features_ns().await?;

    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir)?;
    apply_migrations(&temp_dir, &db_name)?;
    create_branch(&temp_dir, "branch-1")?;
    create_branch_from_ns_db(&temp_dir, "branch-2", ("branches", "branch-1"))?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("branch").arg("remove").arg("branch-1");

    cmd.assert().try_failure().and_then(|assert| {
        assert.try_stderr(predicate::str::contains(
            "Branch branch-1 is used by another branch",
        ))
    })?;

    // Check "branch" record still exist in surrealdb
    let branch: Option<Branch> = get_surrealdb_record(
        "features".to_string(),
        "branching".to_string(),
        "branch".to_string(),
        "branch-1".to_string(),
    )
    .await?;

    ensure!(branch.is_some(), "Branch record should still exist");

    // Check database is still here in surrealdb
    let is_empty =
        is_surreal_db_empty(Some("branches".to_string()), Some("branch-1".to_string())).await?;

    ensure!(!is_empty, "SurrealDB database should not be empty");

    // Check database is still here in surrealdb
    let is_empty = is_surreal_db_empty(
        Some("branches/origin".to_string()),
        Some("branch-1".to_string()),
    )
    .await?;

    ensure!(!is_empty, "SurrealDB database should not be empty");

    temp_dir.close()?;

    Ok(())
}
