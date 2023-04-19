use include_dir::{include_dir, Dir};
use std::{
    io::Write,
    path::{Path, PathBuf},
    process,
};

use crate::{
    cli::ScaffoldTemplate,
    config,
    constants::{EVENTS_DIR_NAME, MIGRATIONS_DIR_NAME, SCHEMAS_DIR_NAME},
};

pub fn main(template: ScaffoldTemplate) {
    let folder_path = config::retrieve_folder_path();

    let schemas_dir_path = concat_path(&folder_path, SCHEMAS_DIR_NAME);
    let events_dir_path = concat_path(&folder_path, EVENTS_DIR_NAME);
    let migrations_dir_path = concat_path(&folder_path, MIGRATIONS_DIR_NAME);

    fails_if_folder_already_exists(&schemas_dir_path, SCHEMAS_DIR_NAME);
    fails_if_folder_already_exists(&events_dir_path, EVENTS_DIR_NAME);
    fails_if_folder_already_exists(&migrations_dir_path, MIGRATIONS_DIR_NAME);

    copy_template_files_to_current_dir(template, folder_path);

    ensures_folder_exists(&schemas_dir_path);
    ensures_folder_exists(&events_dir_path);
    ensures_folder_exists(&migrations_dir_path);

    rename_migrations_files_to_match_current_date(&migrations_dir_path);
}

fn concat_path(folder_path: &Option<String>, dir_name: &str) -> PathBuf {
    match folder_path.to_owned() {
        Some(folder_path) => Path::new(&folder_path).join(dir_name),
        None => Path::new(dir_name).to_path_buf(),
    }
}

fn fails_if_folder_already_exists(dir_path: &PathBuf, dir_name: &str) {
    if dir_path.exists() {
        eprintln!("Error: '{}' folder already exists.", dir_name);
        process::exit(1);
    }
}

fn copy_template_files_to_current_dir(template: ScaffoldTemplate, folder_path: Option<String>) {
    const TEMPLATES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates");

    let template_dir_name = get_template_name(template);
    let from = TEMPLATES_DIR.get_dir(template_dir_name).unwrap();

    let to = match folder_path.to_owned() {
        Some(folder_path) => folder_path,
        None => ".".to_owned(),
    };

    extract(from, to).unwrap();
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

fn ensures_folder_exists(dir_path: &PathBuf) {
    if !dir_path.exists() {
        fs_extra::dir::create(&dir_path, false).unwrap();
    }
}

fn rename_migrations_files_to_match_current_date(migrations_dir_path: &PathBuf) {
    let now = chrono::Local::now();
    let regex = regex::Regex::new(r"^YYYYMMDD_HHMM(\d{2})_").unwrap();

    let migrations_dir = std::fs::read_dir(&migrations_dir_path).unwrap();

    let migration_filenames_to_rename = migrations_dir
        .map(|file| file.unwrap().file_name())
        .filter(|filename| regex.is_match(filename.to_str().unwrap()))
        .collect::<Vec<_>>();

    for filename in migration_filenames_to_rename {
        let captures = regex.captures(filename.to_str().unwrap()).unwrap();
        let seconds = captures.get(1).unwrap().as_str();

        let new_filename_prefix = format!("{}{}_", now.format("%Y%m%d_%H%M"), seconds);
        let new_filename = regex.replace(filename.to_str().unwrap(), new_filename_prefix);

        let from = format!(
            "{}/{}",
            migrations_dir_path.to_str().clone().unwrap(),
            filename.to_str().unwrap()
        );

        let to = format!("{}/{}", migrations_dir_path.to_str().unwrap(), new_filename);

        std::fs::rename(from, to).unwrap();
    }
}
