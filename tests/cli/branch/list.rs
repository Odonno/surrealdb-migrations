use anyhow::Result;
use serial_test::serial;

use crate::helpers::*;

#[tokio::test]
#[serial]
async fn list_existing_branches() -> Result<()> {
    run_with_surreal_instance_async(|| {
        Box::pin(async {
            clear_tests_files()?;
            scaffold_blog_template()?;
            apply_migrations()?;

            create_branch("branch-1")?;
            create_branch("branch-2")?;
            create_branch("branch-3")?;

            let mut cmd = create_cmd()?;

            cmd.arg("branch").arg("list").arg("--no-color");

            let expected =
                " Name     | NS (main) | DB (main) | NS (branch) | DB (branch) | Created at 
----------+-----------+-----------+-------------+-------------+------------
 branch-1 | test      | test      | branches    | branch-1    | just now   
----------+-----------+-----------+-------------+-------------+------------
 branch-2 | test      | test      | branches    | branch-2    | just now   
----------+-----------+-----------+-------------+-------------+------------
 branch-3 | test      | test      | branches    | branch-3    | just now   
\n";

            cmd.assert()
                .try_success()
                .and_then(|assert| assert.try_stdout(expected))?;

            Ok(())
        })
    })
    .await
}

#[tokio::test]
#[serial]
async fn list_with_no_existing_branch() -> Result<()> {
    run_with_surreal_instance_async(|| {
        Box::pin(async {
            clear_tests_files()?;
            scaffold_blog_template()?;
            apply_migrations()?;

            let mut cmd = create_cmd()?;

            cmd.arg("branch").arg("list");

            cmd.assert()
                .try_success()
                .and_then(|assert| assert.try_stdout("There are no branch yet!\n"))?;

            Ok(())
        })
    })
    .await
}
