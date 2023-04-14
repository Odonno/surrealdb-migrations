use assert_cmd::Command;
use serial_test::serial;

use crate::helpers;

#[test]
#[serial]
fn remove_last_migration() {
    helpers::clear_files_dir();

    {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();

        cmd.arg("scaffold").arg("blog");

        cmd.assert().success();
    }

    {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();

        cmd.arg("remove");

        cmd.assert()
            .success()
            .stdout("Migration 'CommentPost' successfully removed\n");
    }
}

#[test]
#[serial]
fn cannot_remove_if_no_migration_file_left() {
    helpers::clear_files_dir();

    {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();

        cmd.arg("scaffold").arg("empty");

        cmd.assert().success();
    }

    {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();

        cmd.arg("remove");

        cmd.assert()
            .failure()
            .stderr("Error: no migration files left\n");
    }
}
