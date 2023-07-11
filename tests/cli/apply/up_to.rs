use anyhow::Result;
use assert_fs::TempDir;

use crate::helpers::*;

#[test]
fn apply_with_skipped_migrations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir)?;

    let first_migration_name = get_first_migration_name(&temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply").arg("--up").arg(first_migration_name);

    cmd.assert().try_success().and_then(|assert| {
        assert.try_stdout(
            "Executing migration AddAdminUser...
Schema files successfully executed!
Event files successfully executed!
Migration files successfully executed!\n",
        )
    })?;

    Ok(())
}
