use anyhow::Result;
use serial_test::serial;

use crate::helpers::*;

#[test]
#[serial]
fn apply_fails_if_both_up_and_down_args_provided() -> Result<()> {
    run_with_surreal_instance(|| {
        clear_tests_files()?;
        scaffold_blog_template()?;

        let first_migration_name = get_first_migration_name()?;

        let mut cmd = create_cmd()?;

        cmd.arg("apply")
            .arg("--up")
            .arg(&first_migration_name)
            .arg("--down")
            .arg(&first_migration_name);

        cmd.assert().try_failure().and_then(|assert| {
            assert.try_stderr(
                "Error: You can\'t specify both `up` and `down` parameters at the same time\n",
            )
        })?;

        Ok(())
    })
}
