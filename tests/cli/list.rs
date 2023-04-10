use assert_cmd::Command;
use chrono::Local;
use serial_test::serial;

use crate::helpers;

#[test]
#[serial]
fn list_empty_migrations() {
    helpers::clear_files_dir();

    let mut child_process = std::process::Command::new("surreal")
        .arg("start")
        .arg("--user")
        .arg("root")
        .arg("--pass")
        .arg("root")
        .arg("memory")
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

    {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();

        cmd.arg("list");

        let result = cmd
            .assert()
            .try_success()
            .and_then(|assert| assert.try_stdout("No migrations applied yet!\n"));

        match result {
            Ok(_) => {}
            Err(error) => {
                child_process.kill().unwrap();
                panic!("{}", error);
            }
        }
    }

    child_process.kill().unwrap();
}

#[test]
#[serial]
fn list_blog_migrations() {
    helpers::clear_files_dir();

    let mut child_process = std::process::Command::new("surreal")
        .arg("start")
        .arg("--user")
        .arg("root")
        .arg("--pass")
        .arg("root")
        .arg("memory")
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

    {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();

        cmd.arg("list").arg("--no-color");

        let date_prefix = now.format("%Y%m%d_%H%M").to_string();

        let expected = format!(
            " Name         | Executed at | File name                          
--------------+-------------+------------------------------------
 AddAdminUser | just now    | {0}01_AddAdminUser.surql 
--------------+-------------+------------------------------------
 AddPost      | just now    | {0}02_AddPost.surql      
--------------+-------------+------------------------------------
 CommentPost  | just now    | {0}03_CommentPost.surql  
\n",
            date_prefix
        );
        println!("{}", expected);

        let result = cmd
            .assert()
            .try_success()
            .and_then(|assert| assert.try_stdout(expected));

        match result {
            Ok(_) => {}
            Err(error) => {
                child_process.kill().unwrap();
                panic!("{}", error);
            }
        }
    }

    child_process.kill().unwrap();
}
