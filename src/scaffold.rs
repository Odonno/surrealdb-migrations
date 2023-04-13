use fs_extra::dir::CopyOptions;
use std::{
    path::{Path, PathBuf},
    process,
};

use crate::{cli::ScaffoldTemplate, config};

pub fn main(template: ScaffoldTemplate) {
    let folder_path = config::retrieve_folder_path();

    const SCHEMAS_DIR_NAME: &str = "schemas";
    let schemas_dir_path = concat_path(&folder_path, SCHEMAS_DIR_NAME);

    const EVENTS_DIR_NAME: &str = "events";
    let events_dir_path = concat_path(&folder_path, EVENTS_DIR_NAME);

    const MIGRATIONS_DIR_NAME: &str = "migrations";
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
        Some(folder_path) => {
            let path = Path::new(&folder_path);
            path.join(dir_name)
        }
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
    let template_dir_name = get_template_dir_name(template);

    let to = match folder_path.to_owned() {
        Some(folder_path) => folder_path,
        None => ".".to_owned(),
    };

    fs_extra::dir::copy(
        template_dir_name,
        to,
        &CopyOptions::new().content_only(true),
    )
    .unwrap();
}

fn get_template_dir_name(template: ScaffoldTemplate) -> String {
    let template_dir_name = get_template_name(template);
    format!("templates/{}", template_dir_name)
}

fn get_template_name(template: ScaffoldTemplate) -> &'static str {
    match template {
        ScaffoldTemplate::Empty => "empty",
        ScaffoldTemplate::Blog => "blog",
        ScaffoldTemplate::Ecommerce => "ecommerce",
    }
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
