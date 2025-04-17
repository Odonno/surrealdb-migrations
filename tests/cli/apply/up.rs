use assert_fs::TempDir;
use color_eyre::eyre::{ensure, Result};
use insta::{assert_snapshot, Settings};
use predicates::prelude::*;

use crate::helpers::*;

#[test]
fn apply_initial_schema_changes() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;
    remove_folder(&temp_dir.join("migrations"))?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply");

    cmd.assert().try_success().and_then(|assert| {
        assert.try_stdout(
            "Schema files successfully executed!
Event files successfully executed!\n",
        )
    })?;

    temp_dir.close()?;

    Ok(())
}

#[test]
fn cannot_apply_if_surreal_instance_not_running() -> Result<()> {
    let temp_dir = TempDir::new()?;

    add_migration_config_file_with_db_address(&temp_dir, "ws://localhost:12345")?;
    scaffold_blog_template(&temp_dir, false)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply");

    cmd.assert().failure().stderr(predicate::str::contains(
        "There was an error processing a remote WS request",
    ));

    temp_dir.close()?;

    Ok(())
}

#[test]
fn apply_new_schema_changes() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;
    remove_folder(&temp_dir.join("migrations"))?;
    apply_migrations(&temp_dir, &db_name)?;
    add_category_schema_file(&temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply");

    cmd.assert().try_success().and_then(|assert| {
        assert.try_stdout(
            "Schema files successfully executed!
Event files successfully executed!\n",
        )
    })?;

    temp_dir.close()?;

    Ok(())
}

#[test]
fn apply_initial_migrations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    println!("{}", db_name);

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply");

    cmd.assert().try_success().and_then(|assert| {
        assert.try_stdout(
            "Executing migration AddAdminUser...
Executing migration AddPost...
Executing migration CommentPost...
Schema files successfully executed!
Event files successfully executed!
Migration files successfully executed!\n",
        )
    })?;

    temp_dir.close()?;

    Ok(())
}

#[test]
fn apply_new_migrations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;

    let first_migration_name = get_first_migration_name(&temp_dir)?;
    apply_migrations_up_to(&temp_dir, &db_name, &first_migration_name)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply");

    cmd.assert().try_success().and_then(|assert| {
        assert.try_stdout(
            "Executing migration AddPost...
Executing migration CommentPost...
Schema files successfully executed!
Event files successfully executed!
Migration files successfully executed!\n",
        )
    })?;

    temp_dir.close()?;

    Ok(())
}

#[test]
fn apply_with_db_configuration() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Admin, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;
    empty_folder(&temp_dir.join("migrations"))?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply")
        .arg("--username")
        .arg("admin")
        .arg("--password")
        .arg("admin")
        .arg("--ns")
        .arg("root")
        .arg("--db")
        .arg(&db_name);

    cmd.assert().try_success().and_then(|assert| {
        assert.try_stdout(
            "Schema files successfully executed!
Event files successfully executed!\n",
        )
    })?;

    temp_dir.close()?;

    Ok(())
}

#[test]
fn apply_should_skip_events_if_no_events_folder() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;
    empty_folder(&temp_dir.join("migrations"))?;
    remove_folder(&temp_dir.join("events"))?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply");

    cmd.assert()
        .try_success()
        .and_then(|assert| assert.try_stdout("Schema files successfully executed!\n"))?;

    temp_dir.close()?;

    Ok(())
}

#[tokio::test]
async fn apply_initial_schema_changes_in_dry_run() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;
    remove_folder(&temp_dir.join("migrations"))?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply").arg("--dry-run");

    cmd.assert()
        .try_success()
        .and_then(|assert| assert.try_stdout(""))?;

    let is_empty = is_surreal_db_empty(None, Some(db_name)).await?;
    ensure!(is_empty, "SurrealDB should be empty");

    temp_dir.close()?;

    Ok(())
}

#[tokio::test]
async fn apply_initial_migrations_in_dry_run() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply").arg("--dry-run");

    cmd.assert()
        .try_success()
        .and_then(|assert| assert.try_stdout(""))?;

    let is_empty = is_surreal_db_empty(None, Some(db_name)).await?;
    ensure!(is_empty, "SurrealDB should be empty");

    temp_dir.close()?;

    Ok(())
}

#[tokio::test]
async fn apply_initial_migrations_in_dry_run_should_fail() -> Result<()> {
    let temp_dir = TempDir::new()?;

    scaffold_empty_template(&temp_dir, false)?;
    add_invalid_schema_file(&temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply").arg("--dry-run");

    cmd.assert().try_failure()?;

    temp_dir.close()?;

    Ok(())
}

#[tokio::test]
async fn apply_initial_migrations_in_dry_run_using_http_engine() -> Result<()> {
    let temp_dir = TempDir::new()?;

    scaffold_blog_template(&temp_dir, false)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply")
        .arg("--dry-run")
        .arg("--address")
        .arg("http://localhost:8000");

    cmd.assert()
        .try_success()
        .and_then(|assert| assert.try_stdout(""))?;

    let is_empty = is_surreal_db_empty(None, None).await?;
    ensure!(is_empty, "SurrealDB should be empty");

    temp_dir.close()?;

    Ok(())
}

#[test]
fn apply_with_inlined_down_files() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;
    inline_down_migration_files(&temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply");

    cmd.assert().try_success().and_then(|assert| {
        assert.try_stdout(
            "Executing migration AddAdminUser...
Executing migration AddPost...
Executing migration CommentPost...
Schema files successfully executed!
Event files successfully executed!
Migration files successfully executed!\n",
        )
    })?;

    temp_dir.close()?;

    Ok(())
}

#[test]
fn should_support_jwks_features() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;
    add_jwks_schema_file(&temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply");

    cmd.assert().try_success().and_then(|assert| {
        assert.try_stdout(
            "Executing migration AddAdminUser...
Executing migration AddPost...
Executing migration CommentPost...
Schema files successfully executed!
Event files successfully executed!
Migration files successfully executed!\n",
        )
    })?;

    temp_dir.close()?;

    Ok(())
}

#[test]
fn apply_and_output_initial_migrations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply").arg("--dry-run").arg("-o");

    cmd.assert().try_success().map(|assert| {
        let out = String::from_utf8_lossy(&assert.get_output().stdout);

        let mut settings = Settings::new();
        settings.add_filter(r"\d{8}_\d{6}", "[timestamp]");

        settings.bind(|| {
            assert_snapshot!(out);
        });
    })?;

    temp_dir.close()?;

    Ok(())
}

#[test]
fn apply_and_output_new_migrations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;

    let first_migration_name = get_first_migration_name(&temp_dir)?;
    apply_migrations_up_to(&temp_dir, &db_name, &first_migration_name)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply").arg("--dry-run").arg("--output");

    cmd.assert().try_success().map(|assert| {
        let out = String::from_utf8_lossy(&assert.get_output().stdout);

        let mut settings = Settings::new();
        settings.add_filter(r"\d{8}_\d{6}", "[timestamp]");

        settings.bind(|| {
            assert_snapshot!(out);
        });
    })?;

    temp_dir.close()?;

    Ok(())
}

#[test]
#[ignore = "references must be applied late"]
fn apply_should_run_computed_table_last() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;
    remove_folder(&temp_dir.join("migrations"))?;
    add_computed_schema_file(&temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply");

    cmd.assert().try_success().and_then(|assert| {
        assert.try_stdout(
            "Schema files successfully executed!
Event files successfully executed!\n",
        )
    })?;

    temp_dir.close()?;

    Ok(())
}

#[test]
fn apply_with_recursive_schema_folders() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;
    remove_folder(&temp_dir.join("events"))?;
    remove_folder(&temp_dir.join("migrations"))?;
    add_category_schema_file(&temp_dir)?;

    let schemas_path = &temp_dir.join("schemas");
    let v2_path = schemas_path.join("v2");
    let file_name = "category.surql";

    create_folder(&v2_path)?;
    move_file(&schemas_path.join(file_name), &v2_path.join(file_name))?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply").arg("--dry-run").arg("--output");

    cmd.assert().try_success().map(|assert| {
        let out = String::from_utf8_lossy(&assert.get_output().stdout);
        assert_snapshot!(out);
    })?;

    temp_dir.close()?;

    Ok(())
}
