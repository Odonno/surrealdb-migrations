use assert_cmd::Command;
use serial_test::serial;

use crate::helpers;

#[test]
#[serial]
fn apply_initial_schema_changes() {
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

    // remove files in folder test-files/migrations
    let migrations_files_dir = std::path::Path::new("tests-files/migrations");

    if migrations_files_dir.exists() {
        std::fs::remove_dir_all(migrations_files_dir).unwrap();
        std::fs::create_dir(migrations_files_dir).unwrap();
    }

    {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();

        cmd.arg("apply");

        let result = cmd.assert().try_success().and_then(|assert| {
            assert.try_stdout(
                "Schema files successfully executed!
Event files successfully executed!
Migration files successfully executed!\n",
            )
        });

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
fn cannot_apply_if_surreal_instance_not_running() {
    helpers::clear_files_dir();

    {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();

        cmd.arg("scaffold").arg("blog");

        cmd.assert().success();
    }

    {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();

        cmd.arg("apply");

        cmd.assert()
            .failure()
            .stderr("There was an error processing a remote WS request\n");
    }
}

#[test]
#[serial]
fn apply_new_schema_changes() {
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

    // remove files in folder test-files/migrations
    let migrations_files_dir = std::path::Path::new("tests-files/migrations");

    if migrations_files_dir.exists() {
        std::fs::remove_dir_all(migrations_files_dir).unwrap();
        std::fs::create_dir(migrations_files_dir).unwrap();
    }

    {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();

        cmd.arg("apply");

        let result = cmd.assert().try_success().and_then(|assert| {
            assert.try_stdout(
                "Schema files successfully executed!
Event files successfully executed!
Migration files successfully executed!\n",
            )
        });

        match result {
            Ok(_) => {}
            Err(error) => {
                child_process.kill().unwrap();
                panic!("{}", error);
            }
        }
    }

    // add new schema file
    let schemas_files_dir = std::path::Path::new("tests-files/schemas");

    if schemas_files_dir.exists() {
        let category_schema_file = schemas_files_dir.join("category.surql");
        const CATEGORY_CONTENT: &str = "DEFINE TABLE category SCHEMALESS;

DEFINE FIELD name ON category TYPE string;
DEFINE FIELD created_at ON comment TYPE datetime VALUE $before OR time::now();";

        std::fs::write(category_schema_file, CATEGORY_CONTENT).unwrap();
    }

    {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();

        cmd.arg("apply");

        let result = cmd.assert().try_success().and_then(|assert| {
            assert.try_stdout(
                "Schema files successfully executed!
Event files successfully executed!
Migration files successfully executed!\n",
            )
        });

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
fn apply_initial_migrations() {
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

        let result = cmd.assert().try_success().and_then(|assert| {
            assert.try_stdout(
                "Schema files successfully executed!
Event files successfully executed!
Executing migration AddAdminUser...
Executing migration AddPost...
Executing migration CommentPost...
Migration files successfully executed!\n",
            )
        });

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
fn apply_with_skipped_migrations() {
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

    {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();

        cmd.arg("apply").arg("--up").arg(first_migration_name);

        let result = cmd.assert().try_success().and_then(|assert| {
            assert.try_stdout(
                "Schema files successfully executed!
Event files successfully executed!
Executing migration AddAdminUser...
Migration files successfully executed!\n",
            )
        });

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
fn apply_new_migrations() {
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

    {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();

        cmd.arg("apply").arg("--up").arg(first_migration_name);

        let result = cmd.assert().try_success().and_then(|assert| {
            assert.try_stdout(
                "Schema files successfully executed!
Event files successfully executed!
Executing migration AddAdminUser...
Migration files successfully executed!\n",
            )
        });

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

        let result = cmd.assert().try_success().and_then(|assert| {
            assert.try_stdout(
                "Schema files successfully executed!
Event files successfully executed!
Executing migration AddPost...
Executing migration CommentPost...
Migration files successfully executed!\n",
            )
        });

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
fn apply_with_db_configuration() {
    helpers::clear_files_dir();

    let mut child_process = std::process::Command::new("surreal")
        .arg("start")
        .arg("--user")
        .arg("admin")
        .arg("--pass")
        .arg("admin")
        .arg("memory")
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

    // remove files in folder test-files/migrations
    let migrations_files_dir = std::path::Path::new("tests-files/migrations");

    if migrations_files_dir.exists() {
        std::fs::remove_dir_all(migrations_files_dir).unwrap();
        std::fs::create_dir(migrations_files_dir).unwrap();
    }

    {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();

        cmd.arg("apply")
            .arg("--username")
            .arg("admin")
            .arg("--password")
            .arg("admin")
            .arg("--ns")
            .arg("namespace")
            .arg("--db")
            .arg("database");

        let result = cmd.assert().try_success().and_then(|assert| {
            assert.try_stdout(
                "Schema files successfully executed!
Event files successfully executed!
Migration files successfully executed!\n",
            )
        });

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
