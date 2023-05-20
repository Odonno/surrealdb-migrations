use anyhow::{anyhow, Result};
use assert_cmd::Command;
use std::{fs, io, path::PathBuf};

pub fn scaffold_empty_template() -> Result<()> {
    scaffold_template("empty")
}

pub fn scaffold_blog_template() -> Result<()> {
    scaffold_template("blog")
}

fn scaffold_template(template_name: &str) -> Result<()> {
    let mut cmd = create_cmd()?;
    cmd.arg("scaffold").arg("template").arg(template_name);
    cmd.assert().try_success()?;

    Ok(())
}

pub fn apply_migrations() -> Result<()> {
    let mut cmd = create_cmd()?;
    cmd.arg("apply");
    cmd.assert().try_success()?;

    Ok(())
}

pub fn apply_migrations_up_to(name: &str) -> Result<()> {
    let mut cmd = create_cmd()?;
    cmd.arg("apply").arg("--up").arg(name);
    cmd.assert().try_success()?;

    Ok(())
}

pub fn create_cmd() -> Result<Command> {
    let cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    Ok(cmd)
}

pub fn get_first_migration_name() -> Result<String> {
    let first_migration_file = get_first_migration_file()?;

    let first_migration_name = first_migration_file
        .file_stem()
        .ok_or_else(|| anyhow!("Could not get file stem"))?
        .to_str()
        .ok_or_else(|| anyhow!("Could not convert file stem to str"))?
        .to_owned();

    Ok(first_migration_name)
}

pub fn get_first_migration_file() -> Result<PathBuf> {
    let migrations_files_dir = std::path::Path::new("tests-files/migrations");

    let mut migration_files = fs::read_dir(migrations_files_dir)?
        .map(|entry| -> io::Result<PathBuf> { Ok(entry?.path()) })
        .collect::<Result<Vec<PathBuf>, io::Error>>()?;

    migration_files.sort_by(|a, b| {
        a.file_name()
            .unwrap_or_default()
            .cmp(b.file_name().unwrap_or_default())
    });

    let first_migration_file = migration_files
        .first()
        .ok_or_else(|| anyhow!("No migration files found"))?;

    Ok(first_migration_file.to_path_buf())
}
