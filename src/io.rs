use color_eyre::eyre::{eyre, ContextCompat, Result, WrapErr};
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
    models::{DefinitionDiff, SchemaMigrationDefinition, ScriptMigration},
};

pub fn concat_path(folder_path: &Option<String>, dir_name: &str) -> PathBuf {
    match folder_path.to_owned() {
        Some(folder_path) => Path::new(&folder_path).join(dir_name),
        None => Path::new(dir_name).to_path_buf(),
    }
}

pub fn can_use_filesystem(config_file: Option<&str>) -> Result<bool> {
    let folder_path = config::retrieve_folder_path(config_file);
    let script_migration_path = concat_path(&folder_path, SCHEMAS_DIR_NAME)
        .join(format!("{}.surql", SCRIPT_MIGRATION_TABLE_NAME));
    let script_migration_file_try_exists = script_migration_path.try_exists().ok();

    let can_use_filesystem = script_migration_file_try_exists.unwrap_or(false);

    Ok(can_use_filesystem)
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

#[cfg(test)]
pub fn create_surql_file(full_name: &str, content: &'static str) -> SurqlFile {
    SurqlFile {
        name: full_name.to_string(),
        full_name: full_name.to_string(),
        content: Box::new(move || Some(content.to_string())),
    }
}

pub fn extract_schemas_files(
    config_file: Option<&str>,
    embedded_dir: Option<&Dir<'static>>,
) -> Result<Vec<SurqlFile>> {
    let dir_path = Path::new(SCHEMAS_DIR_NAME).to_path_buf();
    extract_surql_files(config_file, dir_path, embedded_dir)
}

pub fn extract_events_files(
    config_file: Option<&str>,
    embedded_dir: Option<&Dir<'static>>,
) -> Result<Vec<SurqlFile>> {
    let dir_path = Path::new(EVENTS_DIR_NAME).to_path_buf();
    extract_surql_files(config_file, dir_path, embedded_dir)
}

pub fn extract_forward_migrations_files(
    config_file: Option<&str>,
    embedded_dir: Option<&Dir<'static>>,
) -> Vec<SurqlFile> {
    let root_migrations_dir = Path::new(MIGRATIONS_DIR_NAME).to_path_buf();
    let root_migrations_files =
        match extract_surql_files(config_file, root_migrations_dir, embedded_dir).ok() {
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

pub fn extract_backward_migrations_files(
    config_file: Option<&str>,
    embedded_dir: Option<&Dir<'static>>,
) -> Vec<SurqlFile> {
    let root_migrations_dir = Path::new(MIGRATIONS_DIR_NAME).to_path_buf();
    let root_migrations_files =
        match extract_surql_files(config_file, root_migrations_dir, embedded_dir).ok() {
            Some(files) => files,
            None => vec![],
        };

    let root_backward_migrations_files = root_migrations_files
        .into_iter()
        .filter(|file| file.name.ends_with(".down.surql"))
        .collect::<Vec<_>>();

    let down_migrations_dir = Path::new(MIGRATIONS_DIR_NAME).join(DOWN_MIGRATIONS_DIR_NAME);
    let down_migrations_files =
        match extract_surql_files(config_file, down_migrations_dir, embedded_dir).ok() {
            Some(files) => files,
            None => vec![],
        };

    let mut backward_migrations_files = root_backward_migrations_files;
    backward_migrations_files.extend(down_migrations_files);

    get_sorted_migrations_files(backward_migrations_files)
}

fn extract_surql_files(
    config_file: Option<&str>,
    dir_path: PathBuf,
    embedded_dir: Option<&Dir<'static>>,
) -> Result<Vec<SurqlFile>> {
    match embedded_dir {
        Some(dir) => extract_surql_files_from_embedded_dir(dir_path, dir),
        None => extract_surql_files_from_filesystem(config_file, dir_path),
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

fn extract_surql_files_from_filesystem(
    config_file: Option<&str>,
    dir_path: PathBuf,
) -> Result<Vec<SurqlFile>> {
    let dir_path_str = dir_path.display().to_string();

    let folder_path = config::retrieve_folder_path(config_file);
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
    if let Some(DirEntryValue::Boolean(value)) = f.get(&entry_attribute) {
        return Some(value);
    }
    None
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

pub fn ensures_folder_exists(dir_path: &PathBuf) -> Result<()> {
    if !dir_path.exists() {
        fs_extra::dir::create_all(dir_path, false)?;
    }

    Ok(())
}

pub struct JsonDefinitionFile {
    pub name: String,
    content: Box<dyn Fn() -> Option<String> + Send + Sync>,
}

impl JsonDefinitionFile {
    pub fn get_content(&self) -> Option<String> {
        (self.content)()
    }
}

pub fn extract_json_definition_files(
    config_file: Option<&str>,
    dir_path: &Path,
    embedded_dir: Option<&Dir<'static>>,
) -> Result<Vec<JsonDefinitionFile>> {
    match embedded_dir {
        Some(dir) => extract_json_definition_files_from_embedded_dir(dir_path, dir),
        None => extract_json_definition_files_from_filesystem(config_file, dir_path),
    }
}

fn extract_json_definition_files_from_embedded_dir(
    dir_path: &Path,
    dir: &Dir<'static>,
) -> Result<Vec<JsonDefinitionFile>> {
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

            match (is_file, name) {
                (false, ..) => None,
                (_, Some(name)) => Some(JsonDefinitionFile {
                    name,
                    content: Box::new(move || get_embedded_file_content(f)),
                }),
                _ => None,
            }
        })
        .collect::<Vec<_>>();

    Ok(files)
}

fn extract_json_definition_files_from_filesystem(
    config_file: Option<&str>,
    dir_path: &Path,
) -> Result<Vec<JsonDefinitionFile>> {
    let dir_path_str = dir_path.display().to_string();

    let folder_path = config::retrieve_folder_path(config_file);
    let dir_path = concat_path(&folder_path, &dir_path_str);

    if !dir_path.exists() {
        return Ok(vec![]);
    }

    let mut config = HashSet::new();
    config.insert(DirEntryAttr::Name);
    config.insert(DirEntryAttr::Path);
    config.insert(DirEntryAttr::IsFile);

    let files = fs_extra::dir::ls(dir_path, &config)
        .context(format!("Error listing {} directory", dir_path_str))?
        .items;

    let files = files
        .iter()
        .filter_map(|f| {
            let is_file = extract_boolean_dir_entry_value(f, DirEntryAttr::IsFile);
            let name = extract_string_dir_entry_value(f, DirEntryAttr::Name);
            let path = extract_string_dir_entry_value(f, DirEntryAttr::Path);

            match (is_file, name, path) {
                (None, ..) => None,
                (Some(false), ..) => None,
                (_, Some(name), Some(path)) => {
                    let path = path.clone();

                    Some(JsonDefinitionFile {
                        name: name.to_string(),
                        content: Box::new(move || fs_extra::file::read_to_string(&path).ok()),
                    })
                }
                _ => None,
            }
        })
        .collect::<Vec<_>>();

    Ok(files)
}

pub fn create_definition_files(
    config_file: Option<&str>,
    definitions_path: PathBuf,
    initial_definition_path: PathBuf,
    schema_definitions: String,
    event_definitions: String,
) -> Result<()> {
    let forward_migrations_files = extract_forward_migrations_files(config_file, None);
    let last_migration_file = forward_migrations_files.last();

    if let Some(last_migration_file) = last_migration_file {
        update_migration_definition_file(
            config_file,
            definitions_path,
            initial_definition_path,
            last_migration_file,
            schema_definitions,
            event_definitions,
        )?;
    } else {
        create_initial_definition_file(
            config_file,
            &definitions_path,
            &initial_definition_path,
            schema_definitions,
            event_definitions,
        )?;
    }

    Ok(())
}

fn update_migration_definition_file(
    config_file: Option<&str>,
    definitions_path: PathBuf,
    initial_definition_path: PathBuf,
    last_migration_file: &SurqlFile,
    schema_definitions: String,
    event_definitions: String,
) -> Result<()> {
    let mut definition_files = extract_json_definition_files(config_file, &definitions_path, None)?;
    definition_files.sort_by(|a, b| a.name.cmp(&b.name));
    let definition_files = definition_files;

    let initial_definition_file = definition_files.iter().find(|file| file.name == "_initial");

    let initial_definition_str = match initial_definition_file {
        Some(initial_definition_file) => initial_definition_file.get_content().unwrap_or_default(),
        None => create_initial_definition_file(
            config_file,
            &definitions_path,
            &initial_definition_path,
            schema_definitions.to_string(),
            event_definitions.to_string(),
        )?,
    };

    let folder_path = config::retrieve_folder_path(config_file);
    let definitions_path = match &folder_path {
        Some(folder_path) => Path::new(&folder_path).join(definitions_path),
        None => definitions_path,
    };

    let initial_definition =
        serde_json::from_str::<SchemaMigrationDefinition>(&initial_definition_str)?;

    let definition_diffs = definition_files
        .into_iter()
        .filter(filter_except_initial_definition)
        .filter(|file| file.name < last_migration_file.name)
        .map(|file| file.get_content().unwrap_or_default())
        .collect::<Vec<_>>();

    let last_applied_definition =
        calculate_definition_using_patches(initial_definition, definition_diffs)?;

    let current_definition = SchemaMigrationDefinition {
        schemas: schema_definitions,
        events: event_definitions,
    };

    let definition_filepath = definitions_path.join(format!("{}.json", last_migration_file.name));

    let has_schema_diffs =
        last_applied_definition.schemas.trim() != current_definition.schemas.trim();
    let has_event_diffs = last_applied_definition.events.trim() != current_definition.events.trim();

    let schemas_diffs = match has_schema_diffs {
        true => Some(
            diffy::create_patch(
                &last_applied_definition.schemas,
                &current_definition.schemas,
            )
            .to_string(),
        ),
        false => None,
    };

    let events_diffs = match has_event_diffs {
        true => Some(
            diffy::create_patch(&last_applied_definition.events, &current_definition.events)
                .to_string(),
        ),
        false => None,
    };

    let definition_diff = DefinitionDiff {
        schemas: schemas_diffs,
        events: events_diffs,
    };

    let has_changes = definition_diff.schemas.is_some() || definition_diff.events.is_some();

    match has_changes {
        true => {
            // Create definition file if any changes
            ensures_folder_exists(&definitions_path)?;

            let serialized_definition = serde_json::to_string(&definition_diff)?;
            fs_extra::file::write_all(&definition_filepath, &serialized_definition)?;
        }
        false => {
            // Remove definition file if exists
            let definition_filepath = Path::new(&definition_filepath);

            if definition_filepath.exists() {
                fs_extra::file::remove(definition_filepath)?;
            }
        }
    };

    Ok(())
}

fn create_initial_definition_file(
    config_file: Option<&str>,
    definitions_path: &PathBuf,
    initial_definition_path: &PathBuf,
    schema_definitions: String,
    event_definitions: String,
) -> Result<String> {
    let folder_path = config::retrieve_folder_path(config_file);
    let definitions_path = match &folder_path {
        Some(folder_path) => Path::new(&folder_path).join(definitions_path),
        None => definitions_path.clone(),
    };
    let initial_definition_path = match &folder_path {
        Some(folder_path) => Path::new(&folder_path).join(initial_definition_path),
        None => initial_definition_path.clone(),
    };

    ensures_folder_exists(&definitions_path)?;

    let current_definition = SchemaMigrationDefinition {
        schemas: schema_definitions,
        events: event_definitions,
    };

    let serialized_definition = serde_json::to_string(&current_definition)?;
    fs_extra::file::write_all(initial_definition_path, &serialized_definition)?;

    Ok(serialized_definition)
}

pub fn filter_except_initial_definition(file: &JsonDefinitionFile) -> bool {
    file.name != "_initial"
}

pub fn calculate_definition_using_patches(
    initial_definition: SchemaMigrationDefinition,
    definition_diffs: Vec<String>,
) -> Result<SchemaMigrationDefinition> {
    let mut patched_definition = initial_definition;

    for definition_diff_str in definition_diffs {
        let definition_diff = serde_json::from_str::<DefinitionDiff>(&definition_diff_str)?;

        let schemas = match definition_diff.schemas {
            Some(schemas_diff) => apply_patch(patched_definition.schemas, schemas_diff)?,
            _ => patched_definition.schemas,
        };

        let events = match definition_diff.events {
            Some(events_diff) => apply_patch(patched_definition.events, events_diff)?,
            _ => patched_definition.events,
        };

        patched_definition = SchemaMigrationDefinition { schemas, events };
    }

    Ok(patched_definition)
}

pub fn apply_patch(text: String, diff: String) -> Result<String> {
    let patch = diffy::Patch::from_str(&diff)?;
    let value = diffy::apply(&text, &patch)?;

    Ok(value)
}

pub fn get_current_definition(
    config_file: Option<&str>,
    definitions_path: PathBuf,
    last_migration_applied: &ScriptMigration,
    embedded_dir: Option<&Dir<'static>>,
) -> Result<SchemaMigrationDefinition> {
    let mut definition_files =
        extract_json_definition_files(config_file, &definitions_path, embedded_dir)?;
    definition_files.sort_by(|a, b| a.name.cmp(&b.name));
    let definition_files = definition_files;

    let initial_definition_file = definition_files.iter().find(|file| file.name == "_initial");

    let initial_definition_str = match initial_definition_file {
        Some(initial_definition_file) => initial_definition_file.get_content().unwrap_or_default(),
        None => return Err(eyre!("Initial definition file not found")),
    };

    let initial_definition =
        serde_json::from_str::<SchemaMigrationDefinition>(&initial_definition_str)?;

    let definition_diffs = definition_files
        .into_iter()
        .filter(filter_except_initial_definition)
        .take_while(|file| take_while_applied_or_before(file, last_migration_applied))
        .map(|file| file.get_content().unwrap_or_default())
        .collect::<Vec<_>>();

    let last_applied_definition =
        calculate_definition_using_patches(initial_definition, definition_diffs)?;

    Ok(last_applied_definition)
}

fn take_while_applied_or_before(
    file: &JsonDefinitionFile,
    last_migration_applied: &ScriptMigration,
) -> bool {
    file.name <= last_migration_applied.script_name
}

pub fn get_initial_definition(
    config_file: Option<&str>,
    definitions_path: PathBuf,
    embedded_dir: Option<&Dir<'static>>,
) -> Result<SchemaMigrationDefinition> {
    let definition_str =
        extract_initial_definition_content(config_file, definitions_path, embedded_dir)?;
    let definition = serde_json::from_str::<SchemaMigrationDefinition>(&definition_str)?;

    Ok(definition)
}

fn extract_initial_definition_content(
    config_file: Option<&str>,
    definitions_path: PathBuf,
    embedded_dir: Option<&Dir<'static>>,
) -> Result<String> {
    const INITIAL_DEFINITION_FILENAME: &str = "_initial.json";

    match embedded_dir {
        Some(dir) => {
            let dir_path_str = definitions_path.display().to_string();
            let path = definitions_path.join(INITIAL_DEFINITION_FILENAME);

            let file = dir.get_file(path).context(format!(
                "{} file not found in {} directory",
                INITIAL_DEFINITION_FILENAME, &dir_path_str
            ))?;

            let content = get_embedded_file_content(file).context(format!(
                "Error while reading {} file in {} directory",
                INITIAL_DEFINITION_FILENAME, &dir_path_str
            ))?;

            Ok(content)
        }
        None => {
            let folder_path = config::retrieve_folder_path(config_file);
            let definitions_path = match &folder_path {
                Some(folder_path) => Path::new(&folder_path).join(definitions_path),
                None => definitions_path,
            };

            let initial_definition_filepath = definitions_path.join(INITIAL_DEFINITION_FILENAME);
            let content =
                fs_extra::file::read_to_string(&initial_definition_filepath).wrap_err(format!(
                    "initial_definition_filepath not found at: {:?}",
                    initial_definition_filepath,
                ))?;

            Ok(content)
        }
    }
}

pub fn get_migration_definition_diff(
    config_file: Option<&str>,
    definitions_path: PathBuf,
    migration_name: String,
    embedded_dir: Option<&Dir<'static>>,
) -> Result<Option<DefinitionDiff>> {
    let definition_str = extract_definition_diff_content(
        config_file,
        definitions_path,
        migration_name,
        embedded_dir,
    )?;

    if let Some(definition_str) = definition_str {
        let definition = serde_json::from_str::<DefinitionDiff>(&definition_str)?;
        Ok(Some(definition))
    } else {
        Ok(None)
    }
}

fn extract_definition_diff_content(
    config_file: Option<&str>,
    definitions_path: PathBuf,
    migration_name: String,
    embedded_dir: Option<&Dir<'static>>,
) -> Result<Option<String>> {
    let definition_filename = format!("{}.json", migration_name);

    match embedded_dir {
        Some(dir) => {
            let dir_path_str = definitions_path.display().to_string();
            let path = definitions_path.join(&definition_filename);

            if dir.contains(&path) {
                let file = dir.get_file(&path).context(format!(
                    "{} file not found in {} directory",
                    &definition_filename, &dir_path_str
                ))?;

                let content = get_embedded_file_content(file).context(format!(
                    "Error while reading {} file in {} directory",
                    definition_filename, &dir_path_str
                ))?;

                Ok(Some(content))
            } else {
                Ok(None)
            }
        }
        None => {
            let folder_path = config::retrieve_folder_path(config_file);
            let definitions_path = match &folder_path {
                Some(folder_path) => Path::new(&folder_path).join(definitions_path),
                None => definitions_path,
            };

            let definition_filepath = definitions_path.join(definition_filename);

            if definition_filepath.exists() {
                let content = fs_extra::file::read_to_string(&definition_filepath)?;
                Ok(Some(content))
            } else {
                Ok(None)
            }
        }
    }
}
