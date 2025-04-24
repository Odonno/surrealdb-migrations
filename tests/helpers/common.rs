use assert_cmd::Command;
use color_eyre::eyre::{eyre, ContextCompat, Result};
use lexicmp::natural_lexical_cmp;
use names::{Generator, Name};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub fn scaffold_empty_template(path: &Path, use_traditional_approach: bool) -> Result<()> {
    scaffold_template(None, "empty", path, use_traditional_approach)
}

pub fn scaffold_blog_template(path: &Path, use_traditional_approach: bool) -> Result<()> {
    scaffold_template(None, "blog", path, use_traditional_approach)
}

fn scaffold_template(
    config_file: Option<&str>,
    template_name: &str,
    path: &Path,
    use_traditional_approach: bool,
) -> Result<()> {
    let mut cmd = create_cmd(path)?;

    cmd.arg("scaffold").arg("template").arg(template_name);
    if let Some(config_file) = config_file {
        cmd.arg("--config-file").arg(config_file);
    }
    if use_traditional_approach {
        cmd.arg("--traditional");
    }
    cmd.assert().try_success()?;

    Ok(())
}

pub fn apply_migrations(path: &Path, db: &str) -> Result<()> {
    let mut cmd = create_cmd(path)?;
    cmd.arg("apply").arg("--db").arg(db);
    cmd.assert().try_success()?;

    Ok(())
}

pub fn apply_migrations_up_to(path: &Path, db: &str, name: &str) -> Result<()> {
    let mut cmd = create_cmd(path)?;
    cmd.arg("apply").arg("--up").arg(name).arg("--db").arg(db);
    cmd.assert().try_success()?;

    Ok(())
}

pub fn apply_migrations_down(path: &Path, db: &str, name: &str) -> Result<()> {
    let mut cmd = create_cmd(path)?;
    cmd.arg("apply").arg("--down").arg(name).arg("--db").arg(db);
    cmd.assert().try_success()?;

    Ok(())
}

pub fn apply_migrations_on_branch(path: &Path, branch_name: &str) -> Result<()> {
    let mut cmd = create_cmd(path)?;
    cmd.arg("apply")
        .arg("--ns")
        .arg("branches")
        .arg("--db")
        .arg(branch_name);
    cmd.assert().try_success()?;

    Ok(())
}

pub fn create_branch(path: &Path, branch_name: &str) -> Result<()> {
    let mut cmd = create_cmd(path)?;
    cmd.arg("branch")
        .arg("new")
        .arg(branch_name)
        .arg("--address")
        .arg("http://localhost:8000");
    cmd.assert().try_success()?;

    Ok(())
}

pub fn create_branch_from_ns_db(path: &Path, branch_name: &str, ns_db: (&str, &str)) -> Result<()> {
    let (ns, db) = ns_db;

    let mut cmd = create_cmd(path)?;
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

pub fn create_cmd(path: &Path) -> Result<Command> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.current_dir(path);
    cmd.env("NO_COLOR", "1");

    Ok(cmd)
}

pub fn get_first_migration_name(path: &Path) -> Result<String> {
    get_nth_migration_name(path, 0)
}
pub fn get_first_migration_file(path: &Path) -> Result<PathBuf> {
    get_nth_migration_file(path, 0)
}

pub fn get_second_migration_name(path: &Path) -> Result<String> {
    get_nth_migration_name(path, 1)
}
pub fn get_second_migration_file(path: &Path) -> Result<PathBuf> {
    get_nth_migration_file(path, 1)
}

pub fn get_third_migration_name(path: &Path) -> Result<String> {
    get_nth_migration_name(path, 2)
}

fn get_nth_migration_name(
    path: &Path,
    index: i8,
) -> std::result::Result<String, color_eyre::eyre::Error> {
    let migration_name = get_nth_migration_file(path, index)?
        .file_stem()
        .ok_or_else(|| eyre!("Could not get file stem"))?
        .to_str()
        .ok_or_else(|| eyre!("Could not convert file stem to str"))?
        .to_owned();

    Ok(migration_name)
}

fn get_nth_migration_file(path: &Path, index: i8) -> Result<PathBuf> {
    let migrations_files_dir = path.join("migrations");

    let mut migration_files = fs::read_dir(migrations_files_dir)?
        .map(|entry| -> io::Result<PathBuf> { Ok(entry?.path()) })
        .collect::<Result<Vec<PathBuf>, io::Error>>()?;

    migration_files.sort_by(|a, b| {
        natural_lexical_cmp(
            a.file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default(),
            b.file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default(),
        )
    });

    let nth_migration_file = migration_files
        .get(index as usize)
        .ok_or_else(|| eyre!("No migration files found"))?;

    Ok(nth_migration_file.to_path_buf())
}

pub fn generate_random_db_name() -> Result<String> {
    // TODO : ensure uniqueness? query db to check if name exists and create it?
    // TODO : ensure drop table when done (impl Drop trait)

    let mut generator = Generator::with_naming(Name::Numbered);
    let db_name = generator.next().context("Cannot generate db name")?;

    Ok(db_name)
}
