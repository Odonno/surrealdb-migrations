use anyhow::Result;
use serial_test::serial;

use crate::helpers::common::*;

#[test]
#[serial]
fn remove_last_migration() -> Result<()> {
    clear_files_dir()?;
    scaffold_blog_template()?;

    let mut cmd = create_cmd()?;

    cmd.arg("remove");

    cmd.assert()
        .success()
        .stdout("Migration 'CommentPost' successfully removed\n");

    Ok(())
}

#[test]
#[serial]
fn cannot_remove_if_no_migration_file_left() -> Result<()> {
    clear_files_dir()?;
    scaffold_empty_template()?;

    let mut cmd = create_cmd()?;

    cmd.arg("remove");

    cmd.assert()
        .failure()
        .stderr("Error: No migration files left\n");

    Ok(())
}
