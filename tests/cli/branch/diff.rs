use anyhow::Result;
use assert_fs::TempDir;
use serial_test::serial;

use crate::helpers::*;

#[tokio::test]
#[serial("branches")]
async fn diff_without_changes() -> Result<()> {
    remove_features_ns().await?;

    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, &db_name)?;
    scaffold_blog_template(&temp_dir)?;
    apply_migrations(&temp_dir, &db_name)?;
    create_branch(&temp_dir, "test-branch-without-changes")?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("branch")
        .arg("diff")
        .arg("test-branch-without-changes");

    cmd.assert()
        .try_success()
        .and_then(|assert| assert.try_stdout("No schema changes detected\n"))?;

    Ok(())
}

#[tokio::test]
#[serial("branches")]
async fn diff_with_changes() -> Result<()> {
    remove_features_ns().await?;

    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, &db_name)?;
    scaffold_blog_template(&temp_dir)?;
    apply_migrations(&temp_dir, &db_name)?;
    create_branch(&temp_dir, "test-branch-with-changes")?;
    add_category_schema_file(&temp_dir)?;
    apply_migrations_on_branch(&temp_dir, "test-branch-with-changes")?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("branch")
        .arg("diff")
        .arg("test-branch-with-changes");

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
}

#[tokio::test]
#[serial("branches")]
async fn fails_if_branch_does_not_exist() -> Result<()> {
    remove_features_ns().await?;

    let temp_dir = TempDir::new()?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("branch").arg("diff").arg("void");

    cmd.assert()
        .try_failure()
        .and_then(|assert| assert.try_stderr("Error: Branch void does not exist\n"))?;

    Ok(())
}
