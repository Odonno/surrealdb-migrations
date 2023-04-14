use fs_extra::dir::CopyOptions;
use std::{path::Path, process};

use crate::{cli::ScaffoldTemplate, config};

pub fn main(template: ScaffoldTemplate) {
    let folder_path = config::retrieve_folder_path();

    const SCHEMAS_DIR_NAME: &str = "schemas";
    let schemas_dir_path = match folder_path.to_owned() {
        Some(folder_path) => {
            let path = Path::new(&folder_path);
            path.join(SCHEMAS_DIR_NAME)
        }
        None => Path::new(SCHEMAS_DIR_NAME).to_path_buf(),
    };

    const EVENTS_DIR_NAME: &str = "events";
    let events_dir_path = match folder_path.to_owned() {
        Some(folder_path) => {
            let path = Path::new(&folder_path);
            path.join(EVENTS_DIR_NAME)
        }
        None => Path::new(EVENTS_DIR_NAME).to_path_buf(),
    };

    const MIGRATIONS_DIR_NAME: &str = "migrations";
    let migrations_dir_path = match folder_path.to_owned() {
        Some(folder_path) => {
            let path = Path::new(&folder_path);
            path.join(MIGRATIONS_DIR_NAME)
        }
        None => Path::new(MIGRATIONS_DIR_NAME).to_path_buf(),
    };

    // fails if any folder "schemas", "events", "migrations" already exists
    if schemas_dir_path.exists() {
        eprintln!("Error: 'schemas' folder already exists.");
        process::exit(1);
    }
    if events_dir_path.exists() {
        eprintln!("Error: 'events' folder already exists.");
        process::exit(1);
    }
    if migrations_dir_path.exists() {
        eprintln!("Error: 'migrations' folder already exists.");
        process::exit(1);
    }

    // copy template files to current directory
    let template_dir_name = match template {
        ScaffoldTemplate::Empty => "empty",
        ScaffoldTemplate::Blog => "blog",
        ScaffoldTemplate::Ecommerce => "ecommerce",
    };
    let template_dir_name = format!("templates/{}", template_dir_name);

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

    // ensures folders exists
    if !schemas_dir_path.exists() {
        fs_extra::dir::create(&schemas_dir_path, false).unwrap();
    }
    if !events_dir_path.exists() {
        fs_extra::dir::create(&events_dir_path, false).unwrap();
    }
    if !migrations_dir_path.exists() {
        fs_extra::dir::create(&migrations_dir_path, false).unwrap();
    }

    let migrations_dir = std::fs::read_dir(&migrations_dir_path).unwrap();

    // rename migrations files to match the current date
    let now = chrono::Local::now();
    let regex = regex::Regex::new(r"^YYYYMMDD_HHMM(\d{2})_").unwrap();

    for migration_file in migrations_dir {
        let filename = migration_file.unwrap().file_name();
        let should_rename = regex.is_match(filename.to_str().unwrap());

        if should_rename {
            let captures = regex.captures(filename.to_str().unwrap()).unwrap();

            let seconds = captures.get(1).unwrap().as_str();

            let replace_str = format!("{}{}_", now.format("%Y%m%d_%H%M"), seconds);

            let new_filename = regex.replace(filename.to_str().unwrap(), replace_str);

            std::fs::rename(
                format!(
                    "{}/{}",
                    migrations_dir_path.to_str().clone().unwrap(),
                    filename.to_str().unwrap()
                ),
                format!("{}/{}", migrations_dir_path.to_str().unwrap(), new_filename),
            )
            .unwrap();
        }
    }
}
