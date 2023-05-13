use anyhow::Result;
use chrono::Local;
use serial_test::serial;

use crate::helpers::*;

#[test]
#[serial]
fn list_empty_migrations() -> Result<()> {
    run_with_surreal_instance(|| {
        clear_tests_files()?;
        scaffold_empty_template()?;
        apply_migrations()?;

        let mut cmd = create_cmd()?;
        cmd.arg("list");

        cmd.assert()
            .try_success()
            .and_then(|assert| assert.try_stdout("No migrations applied yet!\n"))?;

        Ok(())
    })
}

#[test]
#[serial]
fn list_blog_migrations() -> Result<()> {
    run_with_surreal_instance(|| {
        let now = Local::now();

        clear_tests_files()?;
        scaffold_blog_template()?;
        apply_migrations()?;

        let mut cmd = create_cmd()?;

        cmd.arg("list").arg("--no-color");

        let date_prefix = now.format("%Y%m%d_%H%M").to_string();

        let expected = format!(
            " Name         | Executed at | File name                          
--------------+-------------+------------------------------------
 AddAdminUser | just now    | {0}01_AddAdminUser.surql 
--------------+-------------+------------------------------------
 AddPost      | just now    | {0}02_AddPost.surql      
--------------+-------------+------------------------------------
 CommentPost  | just now    | {0}03_CommentPost.surql  
\n",
            date_prefix
        );

        cmd.assert()
            .try_success()
            .and_then(|assert| assert.try_stdout(expected))?;

        Ok(())
    })
}
