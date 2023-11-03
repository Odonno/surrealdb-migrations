use assert_fs::TempDir;
use chrono::Local;
use color_eyre::eyre::Result;

use crate::helpers::*;

#[test]
fn list_empty_migrations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_empty_template(&temp_dir)?;
    apply_migrations(&temp_dir, &db_name)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("list");

    cmd.assert()
        .try_success()
        .and_then(|assert| assert.try_stdout("No migrations applied yet!\n"))?;

    Ok(())
}

#[test]
fn list_blog_migrations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    let now = Local::now();

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir)?;
    apply_migrations(&temp_dir, &db_name)?;

    let mut cmd = create_cmd(&temp_dir)?;

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
}
