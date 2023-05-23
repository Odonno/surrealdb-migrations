use anyhow::{Context, Result};
use fs_extra::dir::{DirEntryAttr, DirEntryValue};
use include_dir::Dir;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use crate::{
    config,
    constants::{
        DOWN_MIGRATIONS_DIR_NAME, EVENTS_DIR_NAME, MIGRATIONS_DIR_NAME, SCHEMAS_DIR_NAME,
        SCRIPT_MIGRATION_TABLE_NAME,
    },
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

pub struct SurqlFile {
    pub name: String,
    pub full_name: String,
    content: Box<dyn Fn() -> Option<String> + Send + Sync>,
}

impl SurqlFile {
    pub fn get_content(&self) -> Option<String> {
        (self.content)()
    }
}

pub fn extract_schemas_files(embedded_dir: Option<&Dir<'static>>) -> Result<Vec<SurqlFile>> {
    let dir_path = Path::new(SCHEMAS_DIR_NAME).to_path_buf();
    extract_surql_files(dir_path, embedded_dir)
}

pub fn extract_events_files(embedded_dir: Option<&Dir<'static>>) -> Result<Vec<SurqlFile>> {
    let dir_path = Path::new(EVENTS_DIR_NAME).to_path_buf();
    extract_surql_files(dir_path, embedded_dir)
}

pub fn extract_forward_migrations_files(embedded_dir: Option<&Dir<'static>>) -> Vec<SurqlFile> {
    let root_migrations_dir = Path::new(MIGRATIONS_DIR_NAME).to_path_buf();
    let root_migrations_files = match extract_surql_files(root_migrations_dir, embedded_dir).ok() {
        Some(files) => files,
        None => vec![],
    };

    let root_forward_migrations_files = root_migrations_files
        .into_iter()
        .filter(|file| {
            let is_down_file = is_down_file(file);
            !is_down_file
        })
        .collect::<Vec<_>>();

    let forward_migrations_files = root_forward_migrations_files;

    get_sorted_migrations_files(forward_migrations_files)
}

pub fn extract_backward_migrations_files(embedded_dir: Option<&Dir<'static>>) -> Vec<SurqlFile> {
    let root_migrations_dir = Path::new(MIGRATIONS_DIR_NAME).to_path_buf();
    let root_migrations_files = match extract_surql_files(root_migrations_dir, embedded_dir).ok() {
        Some(files) => files,
        None => vec![],
    };

    let root_backward_migrations_files = root_migrations_files
        .into_iter()
        .filter(|file| file.name.ends_with(".down.surql"))
        .collect::<Vec<_>>();

    let down_migrations_dir = Path::new(MIGRATIONS_DIR_NAME).join(DOWN_MIGRATIONS_DIR_NAME);
    let down_migrations_files = match extract_surql_files(down_migrations_dir, embedded_dir).ok() {
        Some(files) => files,
        None => vec![],
    };

    let mut backward_migrations_files = root_backward_migrations_files;
    backward_migrations_files.extend(down_migrations_files);

    get_sorted_migrations_files(backward_migrations_files)
}

fn extract_surql_files(
    dir_path: PathBuf,
    embedded_dir: Option<&Dir<'static>>,
) -> Result<Vec<SurqlFile>> {
    match embedded_dir {
        Some(dir) => extract_surql_files_from_embedded_dir(dir_path, dir),
        None => extract_surql_files_from_filesystem(dir_path),
    }
}

fn extract_surql_files_from_embedded_dir(
    dir_path: PathBuf,
    dir: &Dir<'static>,
) -> Result<Vec<SurqlFile>> {
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

            match (is_file, name, full_name) {
                (false, ..) => None,
                (_, Some(name), Some(full_name)) => Some(SurqlFile {
                    name,
                    full_name,
                    content: Box::new(move || get_embedded_file_content(f)),
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

            match (is_file, name, full_name, path) {
                (None, ..) => None,
                (Some(false), ..) => None,
                (_, Some(name), Some(full_name), Some(path)) => {
                    let path = path.clone();

                    Some(SurqlFile {
                        name: name.to_string(),
                        full_name: full_name.to_string(),
                        content: Box::new(move || fs_extra::file::read_to_string(&path).ok()),
                    })
                }
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
    if let Some(DirEntryValue::String(value)) = f.get(&entry_attribute) {
        return Some(value);
    }
    None
}

fn is_down_file(file: &SurqlFile) -> bool {
    file.full_name.ends_with(".down.surql")
}

fn get_sorted_migrations_files(migrations_files: Vec<SurqlFile>) -> Vec<SurqlFile> {
    let mut sorted_migrations_files = migrations_files;
    sorted_migrations_files.sort_by(|a, b| a.name.cmp(&b.name));

    sorted_migrations_files
}
