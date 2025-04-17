use assert_fs::TempDir;
use color_eyre::eyre::Result;

use crate::helpers::*;

#[test]
fn apply_with_skipped_migrations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir, true)?;

    let second_migration_name = get_second_migration_name(&temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply").arg("--up").arg(second_migration_name);

    cmd.assert().try_success().and_then(|assert| {
        assert.try_stdout(
            "Executing migration Initial...
Executing migration AddAdminUser...
Migration files successfully executed!\n",
        )
    })?;

    temp_dir.close()?;

    Ok(())
}
