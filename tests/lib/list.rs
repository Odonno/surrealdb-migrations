use assert_cmd::Command;
use chrono::{DateTime, Local};
use serial_test::serial;
use std::process::Stdio;

use crate::helpers;
use surrealdb_migrations::{SurrealdbConfiguration, SurrealdbMigrations};

#[tokio::test]
#[serial]
async fn list_empty_migrations() {
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

        cmd.arg("scaffold").arg("empty");

        let result = cmd.assert().try_success();

        match result {
            Ok(_) => {}
            Err(error) => {
                child_process.kill().unwrap();
                panic!("{}", error);
            }
        }
    }

    {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();

        cmd.arg("apply");

        let result = cmd.assert().try_success();

        match result {
            Ok(_) => {}
            Err(error) => {
                child_process.kill().unwrap();
                panic!("{}", error);
            }
        }
    }

    let configuration = SurrealdbConfiguration::default();
    let result = SurrealdbMigrations::new(configuration).list().await;

    match result {
        Ok(_) => {}
        Err(error) => {
            child_process.kill().unwrap();
            panic!("{}", error);
        }
    }

    let migrations_applied = result.unwrap();

    assert_eq!(migrations_applied.len(), 0);

    child_process.kill().unwrap();
}

#[tokio::test]
#[serial]
async fn list_blog_migrations() {
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

    let now = Local::now();

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

    {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();

        cmd.arg("apply");

        let result = cmd.assert().try_success();

        match result {
            Ok(_) => {}
            Err(error) => {
                child_process.kill().unwrap();
                panic!("{}", error);
            }
        }
    }

    let configuration = SurrealdbConfiguration::default();
    let result = SurrealdbMigrations::new(configuration).list().await;

    match result {
        Ok(_) => {}
        Err(error) => {
            child_process.kill().unwrap();
            panic!("{}", error);
        }
    }

    let migrations_applied = result.unwrap();

    assert_eq!(migrations_applied.len(), 3);

    let date_prefix = now.format("%Y%m%d_%H%M").to_string();

    assert_eq!(
        migrations_applied[0].script_name,
        format!("{}01_AddAdminUser", date_prefix)
    );
    assert_eq!(
        DateTime::parse_from_rfc3339(&migrations_applied[0].executed_at)
            .unwrap()
            .timestamp(),
        now.timestamp()
    );

    assert_eq!(
        migrations_applied[1].script_name,
        format!("{}02_AddPost", date_prefix)
    );
    assert_eq!(
        DateTime::parse_from_rfc3339(&migrations_applied[1].executed_at)
            .unwrap()
            .timestamp(),
        now.timestamp()
    );

    assert_eq!(
        migrations_applied[2].script_name,
        format!("{}03_CommentPost", date_prefix)
    );
    assert_eq!(
        DateTime::parse_from_rfc3339(&migrations_applied[2].executed_at)
            .unwrap()
            .timestamp(),
        now.timestamp()
    );

    child_process.kill().unwrap();
}
