use color_eyre::eyre::{ensure, Result};
use assert_fs::TempDir;
use serial_test::serial;

use crate::helpers::*;

#[tokio::test]
#[serial("branches")]
async fn merge_existing_branch() -> Result<()> {
    remove_features_ns().await?;

    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir)?;

    let first_migration_name = get_first_migration_name(&temp_dir)?;
    apply_migrations_up_to(&temp_dir, &db_name, &first_migration_name)?;

    create_branch(&temp_dir, "test-branch-merge-existing")?;

    apply_migrations_on_branch(&temp_dir, "test-branch-merge-existing")?;

    // Check no data exists in the main branch
    let posts: Vec<Post> =
        get_surrealdb_records("test".to_string(), "test".to_string(), "post".to_string()).await?;

    ensure!(posts.is_empty(), "There should be no post");

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("branch")
        .arg("merge")
        .arg("test-branch-merge-existing")
        .arg("--mode")
        .arg("overwrite")
        .arg("--address")
        .arg("http://localhost:8000");

    cmd.assert().try_success().and_then(|assert| {
        assert.try_stdout("Branch test-branch-merge-existing successfully merged\n")
    })?;

    // Check new data exists in the main branch
    let posts: Vec<Post> =
        get_surrealdb_records("test".to_string(), db_name.to_string(), "post".to_string()).await?;

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
    let is_empty = is_surreal_db_empty(
        Some("branches".to_string()),
        Some("test-branch-merge-existing".to_string()),
    )
    .await?;

    ensure!(is_empty, "SurrealDB branch database should be empty");

    // Check database is removed from surrealdb
    let is_empty = is_surreal_db_empty(
        Some("branches/origin".to_string()),
        Some("test-branch-merge-existing".to_string()),
    )
    .await?;

    ensure!(is_empty, "SurrealDB origin branch database should be empty");

    Ok(())
}

#[tokio::test]
#[serial("branches")]
async fn fails_to_merge_if_branch_does_not_exist() -> Result<()> {
    remove_features_ns().await?;

    let temp_dir = TempDir::new()?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("branch")
        .arg("merge")
        .arg("void")
        .arg("--mode")
        .arg("overwrite");

    cmd.assert()
        .try_failure()
        .and_then(|assert| assert.try_stderr("Error: Branch void does not exist\n"))?;

    Ok(())
}
