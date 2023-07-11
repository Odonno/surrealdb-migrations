use anyhow::Result;
use assert_fs::TempDir;
use serial_test::serial;

use crate::helpers::*;

#[test]
#[serial]
fn apply_fails_if_both_up_and_down_args_provided() -> Result<()> {
    let temp_dir = TempDir::new()?;

    scaffold_blog_template(&temp_dir)?;

    let first_migration_name = get_first_migration_name(&temp_dir)?;

    let mut cmd = create_cmd(&temp_dir)?;

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
}
