use anyhow::Result;
use assert_fs::TempDir;
use cli_table::{format::Border, Cell, ColorChoice, Style, Table};
use serial_test::serial;

use crate::helpers::*;

#[tokio::test]
#[serial("branches")]
async fn list_existing_branches() -> Result<()> {
    remove_features_ns().await?;

    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, &db_name)?;
    scaffold_blog_template(&temp_dir)?;
    apply_migrations(&temp_dir, &db_name)?;

    create_branch(&temp_dir, "branch-1")?;
    create_branch(&temp_dir, "branch-2")?;
    create_branch(&temp_dir, "branch-3")?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("branch").arg("list").arg("--no-color");

    let rows = vec![
        vec![
            "branch-1".cell(),
            "test".cell(),
            db_name.to_string().cell(),
            "branches".cell(),
            "branch-1".cell(),
            "just now".cell(),
        ],
        vec![
            "branch-2".cell(),
            "test".cell(),
            db_name.to_string().cell(),
            "branches".cell(),
            "branch-2".cell(),
            "just now".cell(),
        ],
        vec![
            "branch-3".cell(),
            "test".cell(),
            db_name.cell(),
            "branches".cell(),
            "branch-3".cell(),
            "just now".cell(),
        ],
    ];

    let table = rows
        .table()
        .title(vec![
            "Name".cell().bold(true),
            "NS (main)".cell().bold(true),
            "DB (main)".cell().bold(true),
            "NS (branch)".cell().bold(true),
            "DB (branch)".cell().bold(true),
            "Created at".cell().bold(true),
        ])
        .color_choice(ColorChoice::Never)
        .border(Border::builder().build());

    let expected = table.display()?.to_string();

    cmd.assert()
        .try_success()
        .and_then(|assert| assert.try_stdout(expected + "\n"))?;

    Ok(())
}

#[tokio::test]
#[serial("branches")]
async fn list_with_no_existing_branch() -> Result<()> {
    remove_features_ns().await?;

    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, &db_name)?;
    scaffold_blog_template(&temp_dir)?;
    apply_migrations(&temp_dir, &db_name)?;

    let mut cmd = create_cmd(&temp_dir)?;

    cmd.arg("branch").arg("list");

    cmd.assert()
        .try_success()
        .and_then(|assert| assert.try_stdout("There are no branch yet!\n"))?;

    Ok(())
}
