use std::path::Path;

use anyhow::{ensure, Result};
use serial_test::serial;

use crate::helpers::*;

#[test]
#[serial]
fn replay_migrations_on_clean_db() -> Result<()> {
    run_with_surreal_instance(|| {
        clear_tests_files()?;
        scaffold_blog_template()?;
        apply_migrations()?;

        Ok(())
    })?;

    run_with_surreal_instance(|| {
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

        let definitions_files =
            std::fs::read_dir("tests-files/migrations/definitions")?.filter(|entry| {
                match entry.as_ref() {
                    Ok(entry) => entry.path().is_file(),
                    Err(_) => false,
                }
            });
        ensure!(definitions_files.count() == 1);

        ensure!(Path::new("tests-files/migrations/definitions/_initial.json").exists());

        Ok(())
    })
}
