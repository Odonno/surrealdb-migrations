use anyhow::Result;
use serial_test::serial;

use crate::helpers::*;

#[test]
#[serial]
fn remove_last_migration() -> Result<()> {
    clear_tests_files()?;
    scaffold_blog_template()?;

    let mut cmd = create_cmd()?;

    cmd.arg("remove");

    cmd.assert()
        .success()
        .stdout("Migration 'CommentPost' successfully removed\n");

    let migration_files = std::fs::read_dir("tests-files/migrations")?;
    let migration_files = migration_files.filter(|entry| match entry {
        Ok(entry) => entry.file_type().unwrap().is_file(),
        Err(_) => false,
    });
    assert_eq!(migration_files.count(), 2);

    let down_migration_files = std::fs::read_dir("tests-files/migrations/down")?;
    assert_eq!(down_migration_files.count(), 2);

    Ok(())
}

#[test]
#[serial]
fn cannot_remove_if_no_migration_file_left() -> Result<()> {
    clear_tests_files()?;
    scaffold_empty_template()?;

    let mut cmd = create_cmd()?;

    cmd.arg("remove");

    cmd.assert()
        .failure()
        .stderr("Error: No migration files left\n");

    Ok(())
}
