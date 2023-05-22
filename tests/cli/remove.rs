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

    let migration_files =
        std::fs::read_dir("tests-files/migrations")?.filter(|entry| match entry.as_ref() {
            Ok(entry) => entry.path().is_file(),
            Err(_) => false,
        });

    assert_eq!(migration_files.count(), 2);

    let down_migration_files =
        std::fs::read_dir("tests-files/migrations/down")?.filter(|entry| match entry.as_ref() {
            Ok(entry) => entry.path().is_file(),
            Err(_) => false,
        });
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
