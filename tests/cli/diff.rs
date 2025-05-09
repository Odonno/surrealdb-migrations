use assert_fs::TempDir;
use color_eyre::eyre::{Error, Result};
use insta::{assert_snapshot, Settings};

use crate::helpers::*;

#[test]
fn no_changes_detected() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;
    apply_migrations(&temp_dir, &db_name)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("diff");

    let assert = cmd.assert().try_success()?;
    let stdout = get_stdout_str(assert)?;

    let insta_settings = Settings::new();
    insta_settings.bind(|| {
        assert_snapshot!(stdout);
        Ok::<(), Error>(())
    })?;

    temp_dir.close()?;

    Ok(())
}

#[test]
fn migrations_not_applied() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("diff");

    let assert = cmd.assert().try_success()?;
    let stdout = get_stdout_str(assert)?;

    let insta_settings = Settings::new();
    insta_settings.bind(|| {
        assert_snapshot!(stdout);
        Ok::<(), Error>(())
    })?;

    temp_dir.close()?;

    Ok(())
}

#[tokio::test]
async fn apply_field_changes() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, false)?;
    apply_migrations(&temp_dir, &db_name)?;
    execute_sql_statements(
        "
        DEFINE FIELD OVERWRITE title ON post TYPE string DEFAULT 'Empty title';
        DEFINE FIELD OVERWRITE email ON user TYPE string;
        ",
        DbInstance::Root,
        &db_name,
    )
    .await?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("diff");

    let assert = cmd.assert().try_success()?;
    let stdout = get_stdout_str(assert)?;

    let insta_settings = Settings::new();
    insta_settings.bind(|| {
        assert_snapshot!(stdout);
        Ok::<(), Error>(())
    })?;

    temp_dir.close()?;

    Ok(())
}
