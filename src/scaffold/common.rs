use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Local};
use include_dir::{include_dir, Dir};
use std::{
    io::Write,
    path::{Path, PathBuf},
};

use crate::{
    cli::ScaffoldTemplate,
    constants::{EVENTS_DIR_NAME, MIGRATIONS_DIR_NAME, SCHEMAS_DIR_NAME},
};

pub fn apply_before_scaffold(folder_path: Option<String>) -> Result<()> {
    let schemas_dir_path = concat_path(&folder_path, SCHEMAS_DIR_NAME);
    let events_dir_path = concat_path(&folder_path, EVENTS_DIR_NAME);
    let migrations_dir_path = concat_path(&folder_path, MIGRATIONS_DIR_NAME);

    fails_if_folder_already_exists(&schemas_dir_path, SCHEMAS_DIR_NAME)?;
    fails_if_folder_already_exists(&events_dir_path, EVENTS_DIR_NAME)?;
    fails_if_folder_already_exists(&migrations_dir_path, MIGRATIONS_DIR_NAME)?;

    Ok(())
}

pub fn apply_after_scaffold(folder_path: Option<String>) -> Result<()> {
    let schemas_dir_path = concat_path(&folder_path, SCHEMAS_DIR_NAME);
    let events_dir_path = concat_path(&folder_path, EVENTS_DIR_NAME);
    let migrations_dir_path = concat_path(&folder_path, MIGRATIONS_DIR_NAME);

    ensures_folder_exists(&schemas_dir_path)?;
    ensures_folder_exists(&events_dir_path)?;
    ensures_folder_exists(&migrations_dir_path)?;

    let now = chrono::Local::now();

    rename_migrations_files_to_match_current_date(now, &migrations_dir_path)?;
    rename_down_migrations_files_to_match_current_date(now, &migrations_dir_path)?;

    Ok(())
}

pub fn concat_path(folder_path: &Option<String>, dir_name: &str) -> PathBuf {
    match folder_path.to_owned() {
        Some(folder_path) => Path::new(&folder_path).join(dir_name),
        None => Path::new(dir_name).to_path_buf(),
    }
}

fn fails_if_folder_already_exists(dir_path: &PathBuf, dir_name: &str) -> Result<()> {
    match dir_path.exists() {
        true => Err(anyhow!("'{}' folder already exists.", dir_name)),
        false => Ok(()),
    }
}

pub fn copy_template_files_to_current_dir(
    template: ScaffoldTemplate,
    folder_path: Option<String>,
) -> Result<()> {
    const TEMPLATES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates");

    let template_dir_name = get_template_name(template);
    let from = TEMPLATES_DIR
        .get_dir(template_dir_name)
        .context("Cannot get template dir")?;

    let to = match folder_path.to_owned() {
        Some(folder_path) => folder_path,
        None => ".".to_owned(),
    };

    extract(from, to)?;

    Ok(())
}

fn get_template_name(template: ScaffoldTemplate) -> &'static str {
    match template {
        ScaffoldTemplate::Empty => "empty",
        ScaffoldTemplate::Blog => "blog",
        ScaffoldTemplate::Ecommerce => "ecommerce",
    }
}

// 💡 Function extract customized because it is not implemented in the "include_dir" crate.
// cf. https://github.com/Michael-F-Bryan/include_dir/pull/60
pub fn extract<S: AsRef<Path>>(dir: &Dir<'_>, path: S) -> std::io::Result<()> {
    fn extract_dir<S: AsRef<Path>>(dir: Dir<'_>, path: S) -> std::io::Result<()> {
        let path = path.as_ref();

        for dir in dir.dirs() {
            let dir_path = dir.path().components().skip(1).collect::<PathBuf>();

            std::fs::create_dir_all(path.join(dir_path))?;
            extract_dir(dir.clone(), path)?;
        }

        for file in dir.files() {
            let file_path = file.path().components().skip(1).collect::<PathBuf>();

            let mut fsf = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path.join(file_path))?;
            fsf.write_all(file.contents())?;
            fsf.sync_all()?;
        }

        Ok(())
    }

    extract_dir(dir.clone(), path)
}

fn ensures_folder_exists(dir_path: &PathBuf) -> Result<()> {
    if !dir_path.exists() {
        fs_extra::dir::create_all(&dir_path, false)?;
    }

    Ok(())
}

fn rename_migrations_files_to_match_current_date(
    now: DateTime<Local>,
    migrations_dir_path: &PathBuf,
) -> Result<()> {
    let regex = regex::Regex::new(r"^YYYYMMDD_HHMM(\d{2})_")?;

    let migrations_dir = std::fs::read_dir(&migrations_dir_path)?;

    let migration_filenames_to_rename = migrations_dir
        .filter_map(|entry| match entry {
            Ok(file) => {
                let file_name = file.file_name().to_owned();
                if regex.is_match(file_name.to_str().unwrap_or("")) {
                    Some(file_name)
                } else {
                    None
                }
            }
            Err(_) => None,
        })
        .collect::<Vec<_>>();

    for filename in migration_filenames_to_rename {
        let filename = filename
            .to_str()
            .context("Cannot convert filename to string")?;

        let captures = regex
            .captures(filename)
            .context("Cannot retrieve from pattern")?;
        let seconds = captures
            .get(1)
            .context("Cannot retrieve from pattern")?
            .as_str();

        let new_filename_prefix = format!("{}{}_", now.format("%Y%m%d_%H%M"), seconds);
        let new_filename = regex.replace(filename, new_filename_prefix);

        let from = format!("{}/{}", migrations_dir_path.display(), filename);
        let to = format!("{}/{}", migrations_dir_path.display(), new_filename);

        std::fs::rename(from, to)?;
    }

    Ok(())
}

fn rename_down_migrations_files_to_match_current_date(
    now: DateTime<Local>,
    migrations_dir_path: &PathBuf,
) -> Result<()> {
    let down_migrations_dir_path = migrations_dir_path.join("down");

    if down_migrations_dir_path.exists() {
        rename_migrations_files_to_match_current_date(now, &down_migrations_dir_path)?;
    }

    Ok(())
}
