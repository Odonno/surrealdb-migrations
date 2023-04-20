use assert_cmd::Command;
use serial_test::serial;
use std::process::Stdio;
use surrealdb_migrations::{SurrealdbConfiguration, SurrealdbMigrations};

use crate::helpers;

#[tokio::test]
#[serial]
async fn apply_with_skipped_migrations() {
    helpers::clear_files_dir();

    let mut child_process = std::process::Command::new("surreal")
        .arg("start")
        .arg("--user")
        .arg("root")
        .arg("--pass")
        .arg("root")
        .arg("memory")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();

        cmd.arg("scaffold").arg("blog");

        let result = cmd.assert().try_success();

        match result {
            Ok(_) => {}
            Err(error) => {
                child_process.kill().unwrap();
                panic!("{}", error);
            }
        }
    }

    // read first migration file in folder test-files/migrations
    let migrations_files_dir = std::path::Path::new("tests-files/migrations");

    let migration_files = migrations_files_dir.read_dir().unwrap().collect::<Vec<_>>();
    let mut migration_files = migration_files
        .iter()
        .map(|result| result.as_ref().unwrap().path())
        .collect::<Vec<_>>();
    migration_files.sort_by(|a, b| {
        a.file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .cmp(&b.file_name().unwrap().to_str().unwrap())
    });

    let first_migration_file = &migration_files[0];

    let first_migration_name = first_migration_file.file_stem().unwrap().to_str().unwrap();

    let configuration = SurrealdbConfiguration::default();
    let result = SurrealdbMigrations::new(configuration)
        .up_to(first_migration_name)
        .await;

    match result {
        Ok(_) => {}
        Err(error) => {
            child_process.kill().unwrap();
            panic!("{}", error);
        }
    }

    child_process.kill().unwrap();
}
