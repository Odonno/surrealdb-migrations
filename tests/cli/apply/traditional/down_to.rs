use assert_fs::TempDir;
use color_eyre::eyre::{ensure, Result};

use crate::helpers::*;

#[tokio::test]
async fn apply_revert_all_migrations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, true)?;
    apply_migrations(&temp_dir, &db_name)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply").arg("--down").arg("0");

    cmd.assert().try_success().and_then(|assert| {
        assert.try_stdout(
            "Reverting migration CommentPost...
Reverting migration AddPost...
Reverting migration AddAdminUser...
Migration files successfully executed!\n",
        )
    })?;

    let ns_db = Some(("test", db_name.as_str()));

    let is_table_empty = is_surreal_table_empty(ns_db, "user").await?;
    ensure!(is_table_empty, "'user' table should be empty");

    let is_table_empty = is_surreal_table_empty(ns_db, "post").await?;
    ensure!(is_table_empty, "'post' table should be empty");

    let is_table_empty = is_surreal_table_empty(ns_db, "comment").await?;
    ensure!(is_table_empty, "'comment' table should be empty");

    temp_dir.close()?;

    Ok(())
}

#[tokio::test]
async fn apply_revert_to_first_migration() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, true)?;

    let second_migration_name = get_second_migration_name(&temp_dir)?;

    apply_migrations(&temp_dir, &db_name)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply").arg("--down").arg(second_migration_name);

    cmd.assert().try_success().and_then(|assert| {
        assert.try_stdout(
            "Reverting migration CommentPost...
Reverting migration AddPost...
Migration files successfully executed!\n",
        )
    })?;

    let ns_db = Some(("test", db_name.as_str()));

    let is_table_empty = is_surreal_table_empty(ns_db, "user").await?;
    ensure!(!is_table_empty, "'user' table should not be empty");

    let is_table_empty = is_surreal_table_empty(ns_db, "post").await?;
    ensure!(is_table_empty, "'post' table should be empty");

    let is_table_empty = is_surreal_table_empty(ns_db, "comment").await?;
    ensure!(is_table_empty, "'comment' table should be empty");

    temp_dir.close()?;

    Ok(())
}
