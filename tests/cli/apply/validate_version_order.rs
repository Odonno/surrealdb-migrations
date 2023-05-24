use anyhow::Result;
use serial_test::serial;

use crate::helpers::*;

#[test]
#[serial]
fn fails_if_migrations_applied_with_new_migration_before_last_applied() -> Result<()> {
    run_with_surreal_instance(|| {
        clear_tests_files()?;
        scaffold_blog_template()?;

        let first_migration_file = get_first_migration_file()?;
        std::fs::remove_file(first_migration_file)?;

        apply_migrations()?;

        clear_tests_files()?;
        scaffold_blog_template()?;

        let mut cmd = create_cmd()?;

        cmd.arg("apply").arg("--validate-version-order");

        let first_migration_name = get_first_migration_name()?;

        let error = format!(
            "Error: The following migrations have not been applied: {}\n",
            first_migration_name
        );

        cmd.assert()
            .try_failure()
            .and_then(|assert| assert.try_stderr(error))?;

        Ok(())
    })
}
