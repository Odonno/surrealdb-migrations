use anyhow::{anyhow, Result};
use assert_cmd::Command;
use std::{fs, io, path::PathBuf};

pub fn scaffold_empty_template() -> Result<()> {
    scaffold_template(None, "empty")
}

pub fn scaffold_blog_template() -> Result<()> {
    scaffold_template(None, "blog")
}

fn scaffold_template(config_file: Option<&str>, template_name: &str) -> Result<()> {
    let mut cmd = create_cmd()?;
    cmd.arg("scaffold").arg("template").arg(template_name);
    if let Some(config_file) = config_file {
        cmd.arg("--config-file").arg(config_file);
    }
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

pub fn apply_migrations_down(name: &str) -> Result<()> {
    let mut cmd = create_cmd()?;
    cmd.arg("apply").arg("--down").arg(name);
    cmd.assert().try_success()?;

    Ok(())
}

pub fn apply_migrations_on_branch(branch_name: &str) -> Result<()> {
    let mut cmd = create_cmd()?;
    cmd.arg("apply")
        .arg("--ns")
        .arg("branches")
        .arg("--db")
        .arg(branch_name);
    cmd.assert().try_success()?;

    Ok(())
}

pub fn create_branch(branch_name: &str) -> Result<()> {
    let mut cmd = create_cmd()?;
    cmd.arg("branch")
        .arg("new")
        .arg(branch_name)
        .arg("--address")
        .arg("http://localhost:8000");
    cmd.assert().try_success()?;

    Ok(())
}

pub fn create_branch_from_ns_db(branch_name: &str, ns_db: (&str, &str)) -> Result<()> {
    let (ns, db) = ns_db;

    let mut cmd = create_cmd()?;
    cmd.arg("branch")
        .arg("new")
        .arg(branch_name)
        .arg("--address")
        .arg("http://localhost:8000")
        .arg("--ns")
        .arg(ns)
        .arg("--db")
        .arg(db);
    cmd.assert().try_success()?;

    Ok(())
}

pub fn create_cmd() -> Result<Command> {
    let cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    Ok(cmd)
}

pub fn get_first_migration_name() -> Result<String> {
    get_nth_migration_name(0)
}
pub fn get_first_migration_file() -> Result<PathBuf> {
    get_nth_migration_file(0)
}

pub fn get_second_migration_name() -> Result<String> {
    get_nth_migration_name(1)
}

pub fn get_third_migration_name() -> Result<String> {
    get_nth_migration_name(2)
}

fn get_nth_migration_name(index: i8) -> std::result::Result<String, anyhow::Error> {
    let migration_name = get_nth_migration_file(index)?
        .file_stem()
        .ok_or_else(|| anyhow!("Could not get file stem"))?
        .to_str()
        .ok_or_else(|| anyhow!("Could not convert file stem to str"))?
        .to_owned();

    Ok(migration_name)
}

fn get_nth_migration_file(index: i8) -> Result<PathBuf> {
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
        .get(index as usize)
        .ok_or_else(|| anyhow!("No migration files found"))?;

    Ok(first_migration_file.to_path_buf())
}
