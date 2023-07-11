use anyhow::Result;
use assert_fs::TempDir;

use crate::helpers::*;

#[test]
fn fails_if_migrations_applied_with_new_migration_before_last_applied() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, &db_name)?;
    scaffold_blog_template(&temp_dir)?;

    let first_migration_file = get_first_migration_file(&temp_dir)?;
    std::fs::remove_file(first_migration_file)?;

    apply_migrations(&temp_dir, &db_name)?;

    empty_folder(&temp_dir)?;
    add_migration_config_file_with_db_name(&temp_dir, &db_name)?;
    scaffold_blog_template(&temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply").arg("--validate-version-order");

    let first_migration_name = get_first_migration_name(&temp_dir)?;

    let error = format!(
        "Error: The following migrations have not been applied: {}\n",
        first_migration_name
    );

    cmd.assert()
        .try_failure()
        .and_then(|assert| assert.try_stderr(error))?;

    Ok(())
}
