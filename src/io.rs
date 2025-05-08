use ::surrealdb::sql::{statements::DefineStatement, Query, Statement};
use color_eyre::eyre::{eyre, ContextCompat, Result, WrapErr};
use fs_extra::dir::{DirEntryAttr, DirEntryValue};
use include_dir::Dir;
use itertools::Itertools;
use lexicmp::natural_lexical_cmp;
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use crate::{
    config,
    constants::{
        DOWN_MIGRATIONS_DIR_NAME, DOWN_SURQL_FILE_EXTENSION, DOWN_TAG, EVENTS_DIR_NAME,
        JSON_FILE_EXTENSION, MIGRATIONS_DIR_NAME, ROOT_TAG, SCHEMAS_DIR_NAME,
        SCRIPT_MIGRATION_TABLE_NAME, SURQL_FILE_EXTENSION,
    },
    models::{DefinitionDiff, MigrationDirection, SchemaMigrationDefinition, ScriptMigration},
    surrealdb::parse_statements,
    tags::extract_file_tags,
};

pub fn concat_path(folder_path: &Option<String>, dir_name: &str) -> PathBuf {
    match folder_path.to_owned() {
        Some(folder_path) => Path::new(&folder_path).join(dir_name),
        None => Path::new(dir_name).to_path_buf(),
    }
}

pub fn can_use_filesystem(config_file: Option<&Path>) -> Result<bool> {
    let folder_path = config::retrieve_folder_path(config_file);
    let script_migration_path = concat_path(&folder_path, SCHEMAS_DIR_NAME).join(format!(
        "{}{}",
        SCRIPT_MIGRATION_TABLE_NAME, SURQL_FILE_EXTENSION
    ));
    let script_migration_file_try_exists = script_migration_path.try_exists().ok();

    let can_use_filesystem = script_migration_file_try_exists.unwrap_or(false);

    Ok(can_use_filesystem)
}

pub struct SurqlFile {
    pub name: String,
    pub full_name: String,
    tags: HashSet<String>,
    content: Box<dyn Fn() -> Option<String> + Send + Sync>,
}

impl SurqlFile {
    pub fn get_content(&self) -> Option<String> {
        (self.content)()
    }

    pub fn is_down_file(&self) -> bool {
        self.tags.contains(DOWN_TAG)
    }
}

pub fn concat_files_content(files: &[SurqlFile]) -> String {
    let files_with_content_and_query = files
        .iter()
        .map(|file| {
            let content = file.get_content().unwrap_or_default();
            let query = parse_statements(&content).unwrap_or_default();
            (file, content, query)
        })
        .collect::<Vec<_>>();

    let all_tables = files_with_content_and_query
        .iter()
        .map(|(_, _, query)| query)
        .flat_map(extract_table_names)
        .collect::<HashSet<_>>();

    files_with_content_and_query
        .into_iter()
        .sorted_by(|a, b| {
            let a_query = &a.2;
            let b_query = &b.2;

            // 💡 ensures computed tables are created after the tables they depend
            if consumes_table(a_query, b_query) {
                return Ordering::Greater;
            }
            if consumes_table(b_query, a_query) {
                return Ordering::Less;
            }

            match (
                consumes_any_table(a_query, &all_tables),
                consumes_any_table(b_query, &all_tables),
            ) {
                (true, false) => Ordering::Greater,
                (false, true) => Ordering::Less,
                _ => natural_lexical_cmp(&a.0.name, &b.0.name),
            }
        })
        .map(|(_, content, _)| content)
        .collect::<Vec<_>>()
        .join("\n")
}

fn consumes_table(query1: &Query, query2: &Query) -> bool {
    let query2_table_names = extract_table_names(query2);
    let consumed_table_names = extract_consumed_table_names(query1);

    consumed_table_names
        .intersection(&query2_table_names)
        .count()
        > 0
}

fn consumes_any_table(query: &Query, all_tables: &HashSet<String>) -> bool {
    let consumed_table_names = extract_consumed_table_names(query);

    let query_table_names = extract_table_names(query);
    let other_table_names = all_tables
        .difference(&query_table_names)
        .cloned()
        .collect::<HashSet<_>>();

    other_table_names
        .intersection(&consumed_table_names)
        .count()
        > 0
}

fn extract_table_names(query: &Query) -> HashSet<String> {
    query
        .0
         .0
        .iter()
        .filter_map(|statement| match statement {
            Statement::Define(DefineStatement::Table(table)) => Some(table.name.0.clone()),
            _ => None,
        })
        .collect()
}

fn extract_consumed_table_names(query: &Query) -> HashSet<String> {
    query
        .0
         .0
        .iter()
        .filter_map(|statement| match statement {
            Statement::Define(DefineStatement::Table(table)) => match &table.view {
                Some(view) => {
                    let dependent_tables = view.what.0.iter().map(|t| &t.0).collect::<HashSet<_>>();
                    Some(dependent_tables)
                }
                None => None,
            },
            _ => None,
        })
        .flatten()
        .cloned()
        .collect()
}

pub fn extract_schemas_files(
    config_file: Option<&Path>,
    embedded_dir: Option<&Dir<'static>>,
) -> Result<Vec<SurqlFile>> {
    let dir_path = Path::new(SCHEMAS_DIR_NAME).to_path_buf();
    extract_surql_files(config_file, dir_path, embedded_dir, false)
}

pub fn extract_events_files(
    config_file: Option<&Path>,
    embedded_dir: Option<&Dir<'static>>,
) -> Result<Vec<SurqlFile>> {
    let dir_path = Path::new(EVENTS_DIR_NAME).to_path_buf();
    extract_surql_files(config_file, dir_path, embedded_dir, false)
}

pub fn extract_migrations_files(
    config_file: Option<&Path>,
    embedded_dir: Option<&Dir<'static>>,
    migration_direction: MigrationDirection,
) -> Vec<SurqlFile> {
    let root_migrations_dir = Path::new(MIGRATIONS_DIR_NAME).to_path_buf();
    let root_migrations_files =
        extract_surql_files(config_file, root_migrations_dir, embedded_dir, true)
            .ok()
            .unwrap_or_default();

    let root_migrations_files = root_migrations_files
        .into_iter()
        .filter(|file| match migration_direction {
            MigrationDirection::Forward => !file.is_down_file(),
            MigrationDirection::Backward => file.is_down_file(),
        })
        .collect::<Vec<_>>();

    let forward_migrations_files = root_migrations_files;

    get_sorted_migrations_files(forward_migrations_files)
}

fn extract_surql_files(
    config_file: Option<&Path>,
    dir_path: PathBuf,
    embedded_dir: Option<&Dir<'static>>,
    is_migration_folder: bool,
) -> Result<Vec<SurqlFile>> {
    match embedded_dir {
        Some(dir) => {
            extract_surql_files_from_embedded_dir(dir_path, dir, is_migration_folder, None)
        }
        None => extract_surql_files_from_filesystem(config_file, dir_path),
    }
}

fn extract_surql_files_from_embedded_dir(
    dir_path: PathBuf,
    dir: &Dir<'static>,
    is_migration_folder: bool,
    parent_tags: Option<HashSet<String>>,
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
                (_, Some(name), Some(full_name)) => {
                    let path_str = f.path().to_str().unwrap_or_default();

                    let parent_tags = match &parent_tags {
                        Some(tags) => tags,
                        None => &HashSet::from([ROOT_TAG.into()]),
                    };
                    let file_tags = extract_file_tags(&full_name);

                    let mut tags = file_tags
                        .union(parent_tags)
                        .cloned()
                        .collect::<HashSet<_>>();

                    let is_down_file = full_name.ends_with(DOWN_SURQL_FILE_EXTENSION)
                        || (is_migration_folder
                            && path_str.contains(&format!("{}/", DOWN_MIGRATIONS_DIR_NAME)));

                    if is_down_file {
                        tags.insert(DOWN_TAG.into());
                    }

                    Some(SurqlFile {
                        name,
                        full_name,
                        tags,
                        content: Box::new(move || get_embedded_file_content(f)),
                    })
                }
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
        Some(full_name) => {
            full_name.ends_with(SURQL_FILE_EXTENSION) || full_name.ends_with(JSON_FILE_EXTENSION)
        }
        None => false,
    }
}

fn get_embedded_file_content(f: &include_dir::File) -> Option<String> {
    f.contents_utf8().map(|content| content.to_string())
}

fn extract_surql_files_from_filesystem(
    config_file: Option<&Path>,
    dir_path: PathBuf,
) -> Result<Vec<SurqlFile>> {
    let dir_path_str = dir_path.display().to_string();

    let folder_path = config::retrieve_folder_path(config_file);
    let dir_path = concat_path(&folder_path, &dir_path_str);

    let mut config = HashSet::new();
    config.insert(DirEntryAttr::Name);
    config.insert(DirEntryAttr::Path);
    config.insert(DirEntryAttr::IsFile);
    config.insert(DirEntryAttr::IsDir);
    config.insert(DirEntryAttr::FullName);

    nested_extract_surql_files_from_filesystem(dir_path, &config, None)
        .context(format!("Error listing {} directory", dir_path_str))
}

fn nested_extract_surql_files_from_filesystem(
    dir_path: PathBuf,
    config: &HashSet<DirEntryAttr>,
    parent_tags: Option<&HashSet<String>>,
) -> Option<Vec<SurqlFile>> {
    let Ok(file_result) = fs_extra::dir::ls(dir_path, config) else {
        return None;
    };

    Some(
        file_result
            .items
            .iter()
            .flat_map(|f| {
                let is_dir =
                    extract_boolean_dir_entry_value(f, DirEntryAttr::IsDir).unwrap_or(&false);
                let Some(path) = extract_string_dir_entry_value(f, DirEntryAttr::Path) else {
                    return vec![];
                };

                let Some(full_name) = extract_string_dir_entry_value(f, DirEntryAttr::FullName)
                else {
                    return vec![];
                };

                if *is_dir {
                    let parent_tags = match &parent_tags {
                        Some(tags) => tags,
                        None => &HashSet::from([full_name.into()]),
                    };

                    return nested_extract_surql_files_from_filesystem(
                        path.into(),
                        config,
                        Some(parent_tags),
                    )
                    .unwrap_or_default();
                }

                let is_file =
                    extract_boolean_dir_entry_value(f, DirEntryAttr::IsFile).unwrap_or(&false);
                let name = extract_string_dir_entry_value(f, DirEntryAttr::Name);

                match (is_file, name) {
                    (false, ..) => vec![],
                    (true, Some(name)) if full_name.ends_with(SURQL_FILE_EXTENSION) => {
                        let parent_tags = match &parent_tags {
                            Some(tags) => tags,
                            None => &HashSet::from([ROOT_TAG.into()]),
                        };
                        let file_tags = extract_file_tags(full_name);

                        let tags = file_tags.union(parent_tags).cloned().collect();

                        let path: String = path.clone();

                        vec![SurqlFile {
                            name: name.to_string(),
                            full_name: full_name.to_string(),
                            tags,
                            content: Box::new(move || fs_extra::file::read_to_string(&path).ok()),
                        }]
                    }
                    _ => vec![],
                }
            })
            .collect::<Vec<_>>(),
    )
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

fn get_sorted_migrations_files(migrations_files: Vec<SurqlFile>) -> Vec<SurqlFile> {
    let mut sorted_migrations_files = migrations_files;
    sorted_migrations_files.sort_by(|a, b| natural_lexical_cmp(&a.name, &b.name));

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
    config_file: Option<&Path>,
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
    config_file: Option<&Path>,
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
    config_file: Option<&Path>,
    definitions_path: PathBuf,
    initial_definition_path: PathBuf,
    schema_definitions: String,
    event_definitions: String,
) -> Result<()> {
    let forward_migrations_files =
        extract_migrations_files(config_file, None, MigrationDirection::Forward);
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
    config_file: Option<&Path>,
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
    config_file: Option<&Path>,
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
    config_file: Option<&Path>,
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
    config_file: Option<&Path>,
    definitions_path: PathBuf,
    embedded_dir: Option<&Dir<'static>>,
) -> Result<SchemaMigrationDefinition> {
    let definition_str =
        extract_initial_definition_content(config_file, definitions_path, embedded_dir)?;
    let definition = serde_json::from_str::<SchemaMigrationDefinition>(&definition_str)?;

    Ok(definition)
}

fn extract_initial_definition_content(
    config_file: Option<&Path>,
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
    config_file: Option<&Path>,
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
    config_file: Option<&Path>,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_surql_file(full_name: &str, content: &'static str) -> SurqlFile {
        SurqlFile {
            name: full_name.to_string(),
            full_name: full_name.to_string(),
            tags: HashSet::from([ROOT_TAG.into(), DOWN_TAG.into()]),
            content: Box::new(move || Some(content.to_string())),
        }
    }

    #[test]
    fn concat_empty_list_of_files() {
        let result = concat_files_content(&[]);
        assert_eq!(result, "");
    }

    #[test]
    fn concat_files_in_alphabetic_order() {
        let files = vec![
            create_surql_file("a.text", "Text of a file"),
            create_surql_file("c.text", "Text of c file"),
            create_surql_file("b.text", "Text of b file"),
        ];

        let result = concat_files_content(&files);
        assert_eq!(result, "Text of a file\nText of b file\nText of c file");
    }
}
