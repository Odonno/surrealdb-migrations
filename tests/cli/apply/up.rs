use anyhow::{ensure, Result};
use serial_test::serial;

use crate::helpers::*;

#[test]
#[serial]
fn apply_initial_schema_changes() -> Result<()> {
    run_with_surreal_instance(|| {
        clear_tests_files()?;
        scaffold_blog_template()?;
        remove_folder("tests-files/migrations")?;

        let mut cmd = create_cmd()?;

        cmd.arg("apply");

        cmd.assert().try_success().and_then(|assert| {
            assert.try_stdout(
                "Schema files successfully executed!
Event files successfully executed!
Migration files successfully executed!\n",
            )
        })?;

        Ok(())
    })
}

#[test]
#[serial]
fn cannot_apply_if_surreal_instance_not_running() -> Result<()> {
    clear_tests_files()?;
    scaffold_blog_template()?;

    let mut cmd = create_cmd()?;

    cmd.arg("apply");

    cmd.assert().failure().stderr(
        "Error: There was an error processing a remote WS request

Caused by:
    There was an error processing a remote WS request\n",
    );

    Ok(())
}

#[test]
#[serial]
fn apply_new_schema_changes() -> Result<()> {
    run_with_surreal_instance(|| {
        clear_tests_files()?;
        scaffold_blog_template()?;
        empty_folder("tests-files/migrations")?;
        apply_migrations()?;
        add_new_schema_file()?;

        let mut cmd = create_cmd()?;

        cmd.arg("apply");

        cmd.assert().try_success().and_then(|assert| {
            assert.try_stdout(
                "Schema files successfully executed!
Event files successfully executed!
Migration files successfully executed!\n",
            )
        })?;

        Ok(())
    })
}

#[test]
#[serial]
fn apply_initial_migrations() -> Result<()> {
    run_with_surreal_instance(|| {
        clear_tests_files()?;
        scaffold_blog_template()?;

        let mut cmd = create_cmd()?;

        cmd.arg("apply");

        cmd.assert().try_success().and_then(|assert| {
            assert.try_stdout(
                "Schema files successfully executed!
Event files successfully executed!
Executing migration AddAdminUser...
Executing migration AddPost...
Executing migration CommentPost...
Migration files successfully executed!\n",
            )
        })?;

        Ok(())
    })
}

#[test]
#[serial]
fn apply_new_migrations() -> Result<()> {
    run_with_surreal_instance(|| {
        clear_tests_files()?;
        scaffold_blog_template()?;

        let first_migration_name = get_first_migration_name()?;
        apply_migrations_up_to(&first_migration_name)?;

        let mut cmd = create_cmd()?;

        cmd.arg("apply");

        cmd.assert().try_success().and_then(|assert| {
            assert.try_stdout(
                "Schema files successfully executed!
Event files successfully executed!
Executing migration AddPost...
Executing migration CommentPost...
Migration files successfully executed!\n",
            )
        })?;

        Ok(())
    })
}

#[test]
#[serial]
fn apply_with_db_configuration() -> Result<()> {
    run_with_surreal_instance_with_admin_user(|| {
        clear_tests_files()?;
        scaffold_blog_template()?;
        empty_folder("tests-files/migrations")?;

        let mut cmd = create_cmd()?;

        cmd.arg("apply")
            .arg("--username")
            .arg("admin")
            .arg("--password")
            .arg("admin")
            .arg("--ns")
            .arg("namespace")
            .arg("--db")
            .arg("database");

        cmd.assert().try_success().and_then(|assert| {
            assert.try_stdout(
                "Schema files successfully executed!
Event files successfully executed!
Migration files successfully executed!\n",
            )
        })?;

        Ok(())
    })
}

#[test]
#[serial]
fn apply_should_skip_events_if_no_events_folder() -> Result<()> {
    run_with_surreal_instance(|| {
        clear_tests_files()?;
        scaffold_blog_template()?;
        empty_folder("tests-files/migrations")?;
        remove_folder("tests-files/events")?;

        let mut cmd = create_cmd()?;

        cmd.arg("apply");

        cmd.assert().try_success().and_then(|assert| {
            assert.try_stdout(
                "Schema files successfully executed!
Migration files successfully executed!\n",
            )
        })?;

        Ok(())
    })
}

#[tokio::test]
#[serial]
async fn apply_initial_schema_changes_in_dry_run() -> Result<()> {
    run_with_surreal_instance_async(|| {
        Box::pin(async {
            clear_tests_files()?;
            scaffold_blog_template()?;
            remove_folder("tests-files/migrations")?;

            let mut cmd = create_cmd()?;

            cmd.arg("apply").arg("--dry-run");

            cmd.assert()
                .try_success()
                .and_then(|assert| assert.try_stdout(""))?;

            let is_empty = is_surreal_db_empty(None, None).await?;
            ensure!(is_empty, "SurrealDB should be empty");

            Ok(())
        })
    })
    .await
}

#[tokio::test]
#[serial]
async fn apply_initial_migrations_in_dry_run() -> Result<()> {
    run_with_surreal_instance_async(|| {
        Box::pin(async {
            clear_tests_files()?;
            scaffold_blog_template()?;

            let mut cmd = create_cmd()?;

            cmd.arg("apply").arg("--dry-run");

            cmd.assert()
                .try_success()
                .and_then(|assert| assert.try_stdout(""))?;

            let is_empty = is_surreal_db_empty(None, None).await?;
            ensure!(is_empty, "SurrealDB should be empty");

            Ok(())
        })
    })
    .await
}

#[test]
#[serial]
fn apply_with_inlined_down_files() -> Result<()> {
    run_with_surreal_instance(|| {
        clear_tests_files()?;
        scaffold_blog_template()?;
        inline_down_migration_files()?;

        let mut cmd = create_cmd()?;

        cmd.arg("apply");

        cmd.assert().try_success().and_then(|assert| {
            assert.try_stdout(
                "Schema files successfully executed!
Event files successfully executed!
Executing migration AddAdminUser...
Executing migration AddPost...
Executing migration CommentPost...
Migration files successfully executed!\n",
            )
        })?;

        Ok(())
    })
}
