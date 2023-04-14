use fs_extra::dir::{DirEntryAttr, DirEntryValue};
use std::{collections::HashSet, path::Path, process};

use crate::{config, constants::MIGRATIONS_DIR_NAME};

pub fn main() {
    let folder_path = config::retrieve_folder_path();

    let migrations_path = match folder_path.to_owned() {
        Some(folder_path) => Path::new(&folder_path).join(MIGRATIONS_DIR_NAME),
        None => Path::new(MIGRATIONS_DIR_NAME).to_path_buf(),
    };

    let mut config = HashSet::new();
    config.insert(DirEntryAttr::Name);
    config.insert(DirEntryAttr::Path);
    config.insert(DirEntryAttr::FullName);

    let migrations_files = fs_extra::dir::ls(&migrations_path, &config).unwrap();

    if migrations_files.items.is_empty() {
        eprintln!("Error: no migration files left");
        process::exit(1);
    }

    // get last migration in migrations folder
    let mut sorted_migrations_files = migrations_files.items.iter().collect::<Vec<_>>();
    sorted_migrations_files.sort_by(|a, b| {
        let a = a.get(&DirEntryAttr::Name).unwrap();
        let b = b.get(&DirEntryAttr::Name).unwrap();

        let a = match a {
            DirEntryValue::String(a) => a,
            _ => {
                eprintln!("Cannot get name to migration files");
                process::exit(1);
            }
        };

        let b = match b {
            DirEntryValue::String(b) => b,
            _ => {
                eprintln!("Cannot get name to migration files");
                process::exit(1);
            }
        };

        a.cmp(b)
    });

    let last_migration = sorted_migrations_files.last().unwrap();

    let last_migration_filename = match last_migration.get(&DirEntryAttr::Name) {
        Some(DirEntryValue::String(last_migration_filename)) => last_migration_filename,
        _ => {
            eprintln!("Cannot get name to migration files");
            process::exit(1);
        }
    };

    let last_migration_fullname = match last_migration.get(&DirEntryAttr::FullName) {
        Some(DirEntryValue::String(last_migration_filename)) => last_migration_filename,
        _ => {
            eprintln!("Cannot get name to migration files");
            process::exit(1);
        }
    };

    let last_migration_display_name = last_migration_filename
        .split("_")
        .skip(2)
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .join("_");

    // remove migration file
    let migration_file = migrations_path.join(last_migration_fullname);
    std::fs::remove_file(migration_file).unwrap();

    // remove definition file if exists
    let migration_definition_file_path = Path::new(&migrations_path)
        .join("definitions")
        .join(format!("{}.json", last_migration_filename));

    if migration_definition_file_path.exists() {
        std::fs::remove_file(migration_definition_file_path).unwrap();
    }

    println!(
        "Migration '{}' successfully removed",
        last_migration_display_name
    );
}
