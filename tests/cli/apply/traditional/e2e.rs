use assert_fs::TempDir;
use color_eyre::eyre::{ensure, Result};

use crate::helpers::*;

#[test]
fn replay_migrations_on_clean_db() -> Result<()> {
    let temp_dir = TempDir::new()?;

    scaffold_blog_template(&temp_dir, true)?;

    {
        let db_name = generate_random_db_name()?;

        add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
        apply_migrations(&temp_dir, &db_name)?;
    }

    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("apply");

    cmd.assert().try_success().and_then(|assert| {
        assert.try_stdout(
            "Executing migration Initial...
Executing migration AddAdminUser...
Executing migration AddPost...
Executing migration CommentPost...
Migration files successfully executed!\n",
        )
    })?;

    let migrations_dir = temp_dir.join("migrations");
    let definitions_dir = migrations_dir.join("definitions");

    ensure!(
        !definitions_dir.exists(),
        "Migration definition should not exist"
    );

    temp_dir.close()?;

    Ok(())
}
