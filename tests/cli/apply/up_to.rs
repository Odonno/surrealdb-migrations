use anyhow::Result;
use serial_test::serial;

use crate::helpers::*;

#[test]
#[serial]
fn apply_with_skipped_migrations() -> Result<()> {
    run_with_surreal_instance(|| {
        clear_tests_files()?;
        scaffold_blog_template()?;

        let first_migration_name = get_first_migration_name()?;

        let mut cmd = create_cmd()?;

        cmd.arg("apply").arg("--up").arg(first_migration_name);

        cmd.assert().try_success().and_then(|assert| {
            assert.try_stdout(
                "Schema files successfully executed!
Event files successfully executed!
Executing migration AddAdminUser...
Migration files successfully executed!\n",
            )
        })?;

        Ok(())
    })
}
