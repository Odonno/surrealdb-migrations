use assert_fs::TempDir;
use color_eyre::eyre::Result;
use insta::assert_snapshot;

use crate::helpers::*;

#[test]
fn apply_only_v1_schema() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;
    remove_folder(&temp_dir.join("migrations"))?;
    remove_folder(&temp_dir.join("events"))?;

    let schemas_dir = &temp_dir.join("schemas");
    let v1_dir = schemas_dir.join("v1");
    let v2_dir = schemas_dir.join("v2");

    create_folder(&v1_dir)?;
    create_folder(&v2_dir)?;

    move_file(&schemas_dir.join("post.surql"), &v1_dir.join("post.surql"))?;
    move_file(
        &schemas_dir.join("comment.surql"),
        &v2_dir.join("comment.surql"),
    )?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply")
        .arg("--tags")
        .arg("v1")
        .arg("--dry-run")
        .arg("--output");

    let assert = cmd.assert().try_success()?;

    let stdout = get_stdout_str(assert)?;
    assert_snapshot!(stdout);

    temp_dir.close()?;

    Ok(())
}

#[test]
fn exclude_old_by_default() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;
    remove_folder(&temp_dir.join("migrations"))?;
    remove_folder(&temp_dir.join("events"))?;

    let schemas_dir = &temp_dir.join("schemas");
    let old_dir = schemas_dir.join("old");

    create_folder(&old_dir)?;

    move_file(
        &schemas_dir.join("comment.surql"),
        &old_dir.join("comment.surql"),
    )?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply").arg("--dry-run").arg("--output");

    let assert = cmd.assert().try_success()?;

    let stdout = get_stdout_str(assert)?;
    assert_snapshot!(stdout);

    temp_dir.close()?;

    Ok(())
}

#[test]
fn exclude_v2_schema() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;
    remove_folder(&temp_dir.join("migrations"))?;
    remove_folder(&temp_dir.join("events"))?;

    let schemas_dir = &temp_dir.join("schemas");
    let v2_dir = schemas_dir.join("v2");

    create_folder(&v2_dir)?;

    move_file(
        &schemas_dir.join("comment.surql"),
        &v2_dir.join("comment.surql"),
    )?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply")
        .arg("--exclude-tags")
        .arg("v2")
        .arg("--dry-run")
        .arg("--output");

    let assert = cmd.assert().try_success()?;

    let stdout = get_stdout_str(assert)?;
    assert_snapshot!(stdout);

    temp_dir.close()?;

    Ok(())
}
