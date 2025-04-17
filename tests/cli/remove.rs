use assert_fs::TempDir;
use color_eyre::Result;
use predicates::prelude::*;

use crate::helpers::*;

#[test]
fn remove_last_migration() -> Result<()> {
    let temp_dir = TempDir::new()?;

    scaffold_blog_template(&temp_dir, false)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("remove");

    cmd.assert()
        .success()
        .stdout("Migration 'CommentPost' successfully removed\n");

    let migrations_dir = temp_dir.path().join("migrations");

    let migration_files =
        std::fs::read_dir(&migrations_dir)?.filter(|entry| match entry.as_ref() {
            Ok(entry) => entry.path().is_file(),
            Err(_) => false,
        });

    assert_eq!(migration_files.count(), 2);

    let migrations_down_dir = migrations_dir.join("down");

    let down_migration_files =
        std::fs::read_dir(migrations_down_dir)?.filter(|entry| match entry.as_ref() {
            Ok(entry) => entry.path().is_file(),
            Err(_) => false,
        });
    assert_eq!(down_migration_files.count(), 2);

    temp_dir.close()?;

    Ok(())
}

#[test]
fn cannot_remove_if_no_migration_file_left() -> Result<()> {
    let temp_dir = TempDir::new()?;

    scaffold_empty_template(&temp_dir, false)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("remove");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No migration files left"));

    temp_dir.close()?;

    Ok(())
}
