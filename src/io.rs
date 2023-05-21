use anyhow::{Context, Result};
use fs_extra::dir::{DirEntryAttr, DirEntryValue};
use include_dir::Dir;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use crate::{
    config,
    constants::{SCHEMAS_DIR_NAME, SCRIPT_MIGRATION_TABLE_NAME},
};

pub fn concat_path(folder_path: &Option<String>, dir_name: &str) -> PathBuf {
    match folder_path.to_owned() {
        Some(folder_path) => Path::new(&folder_path).join(dir_name),
        None => Path::new(dir_name).to_path_buf(),
    }
}

pub fn can_use_filesystem() -> bool {
    let folder_path = config::retrieve_folder_path();
    let script_migration_path = concat_path(&folder_path, SCHEMAS_DIR_NAME)
        .join(format!("{}.surql", SCRIPT_MIGRATION_TABLE_NAME));
    let script_migration_file_try_exists = script_migration_path.try_exists().ok();

    script_migration_file_try_exists.unwrap_or(false)
}

#[derive(Debug)]
pub struct SurqlFile {
    pub name: String,
    pub full_name: String,
    pub content: String,
}

pub fn extract_surql_files(
    dir_path: PathBuf,
    embedded_dir: Option<&Dir>,
) -> Result<Vec<SurqlFile>> {
    match embedded_dir {
        Some(dir) => extract_surql_files_from_embedded_dir(dir_path, dir),
        None => extract_surql_files_from_filesystem(dir_path),
    }
}

fn extract_surql_files_from_embedded_dir(dir_path: PathBuf, dir: &Dir) -> Result<Vec<SurqlFile>> {
    let dir_path_str = dir_path.display().to_string();

    let dir = dir
        .get_dir(&dir_path_str)
        .context(format!("{} directory not found", &dir_path_str))?;

    let files = dir
        .files()
        .filter_map(|f| {
            let name = get_embedded_file_name(f);
            let full_name = get_embedded_file_full_name(f);
            let is_file = get_embedded_file_is_file(&full_name);
            let content = get_embedded_file_content(f);

            match (is_file, name, full_name, content) {
                (false, ..) => None,
                (_, Some(name), Some(full_name), Some(content)) => Some(SurqlFile {
                    name,
                    full_name,
                    content,
                }),
                _ => None,
            }
        })
        .collect::<Vec<_>>();

    Ok(files)
}

fn get_embedded_file_name(f: &include_dir::File) -> Option<String> {
    let name = f.path().file_stem();
    let name = match name {
        Some(name) if name.to_str().map(|n| n.ends_with(".down")) == Some(true) => {
            Path::new(name).file_stem()
        }
        Some(name) => Some(name),
        None => None,
    };

    name.and_then(|name| name.to_str())
        .map(|name| name.to_string())
}

fn get_embedded_file_full_name(f: &include_dir::File) -> Option<String> {
    let full_name = f
        .path()
        .file_name()
        .and_then(|full_name| full_name.to_str())
        .map(|full_name| full_name.to_string());
    full_name
}

fn get_embedded_file_is_file(full_name: &Option<String>) -> bool {
    match full_name {
        Some(full_name) => full_name.ends_with(".surql"),
        None => false,
    }
}

fn get_embedded_file_content(f: &include_dir::File) -> Option<String> {
    f.contents_utf8().map(|content| content.to_string())
}

fn extract_surql_files_from_filesystem(dir_path: PathBuf) -> Result<Vec<SurqlFile>> {
    let dir_path_str = dir_path.display().to_string();

    let folder_path = config::retrieve_folder_path();
    let dir_path = concat_path(&folder_path, &dir_path_str);

    let mut config = HashSet::new();
    config.insert(DirEntryAttr::Name);
    config.insert(DirEntryAttr::Path);
    config.insert(DirEntryAttr::IsFile);
    config.insert(DirEntryAttr::FullName);

    let files = fs_extra::dir::ls(dir_path, &config)
        .context(format!("Error listing {} directory", dir_path_str))?
        .items;

    let files = files
        .iter()
        .filter_map(|f| {
            let is_file = extract_boolean_dir_entry_value(f, DirEntryAttr::IsFile);
            let name = extract_string_dir_entry_value(f, DirEntryAttr::Name);
            let full_name = extract_string_dir_entry_value(f, DirEntryAttr::FullName);
            let path = extract_string_dir_entry_value(f, DirEntryAttr::Path);

            let content = match path {
                Some(path) => fs_extra::file::read_to_string(path).ok(),
                None => None,
            };

            match (is_file, name, full_name, content) {
                (None, ..) => None,
                (Some(false), ..) => None,
                (_, Some(name), Some(full_name), Some(content)) => Some(SurqlFile {
                    name: name.to_string(),
                    full_name: full_name.to_string(),
                    content,
                }),
                _ => None,
            }
        })
        .collect::<Vec<_>>();

    Ok(files)
}

fn extract_boolean_dir_entry_value(
    f: &HashMap<DirEntryAttr, DirEntryValue>,
    entry_attribute: DirEntryAttr,
) -> Option<&bool> {
    match f.get(&entry_attribute) {
        Some(DirEntryValue::Boolean(value)) => Some(value),
        _ => None,
    }
}

fn extract_string_dir_entry_value(
    f: &HashMap<DirEntryAttr, DirEntryValue>,
    entry_attribute: DirEntryAttr,
) -> Option<&String> {
    match f.get(&entry_attribute) {
        Some(DirEntryValue::String(value)) => Some(value),
        _ => None,
    }
}
