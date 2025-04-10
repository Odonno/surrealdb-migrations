use ::surrealdb::sql::statements::{
    DefineAccessStatement, DefineEventStatement, DefineFieldStatement, DefineIndexStatement,
    DefineStatement, DefineTableStatement, RemoveAccessStatement, RemoveEventStatement,
    RemoveFieldStatement, RemoveIndexStatement, RemoveStatement, RemoveTableStatement,
};
use chrono::{DateTime, Local};
use color_eyre::eyre::{eyre, ContextCompat, Result};
use include_dir::{include_dir, Dir};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use crate::{
    cli::ScaffoldTemplate,
    constants::{DOWN_MIGRATIONS_DIR_NAME, EVENTS_DIR_NAME, MIGRATIONS_DIR_NAME, SCHEMAS_DIR_NAME},
    io::{self, ensures_folder_exists},
    surrealdb::parse_statements,
};

pub fn apply_before_scaffold(folder_path: Option<String>) -> Result<()> {
    let schemas_dir_path = io::concat_path(&folder_path, SCHEMAS_DIR_NAME);
    let events_dir_path = io::concat_path(&folder_path, EVENTS_DIR_NAME);
    let migrations_dir_path = io::concat_path(&folder_path, MIGRATIONS_DIR_NAME);

    fails_if_folder_already_exists(&schemas_dir_path, SCHEMAS_DIR_NAME)?;
    fails_if_folder_already_exists(&events_dir_path, EVENTS_DIR_NAME)?;
    fails_if_folder_already_exists(&migrations_dir_path, MIGRATIONS_DIR_NAME)?;

    Ok(())
}

pub fn apply_after_scaffold(
    config_file: Option<&Path>,
    traditional: bool,
    folder_path: Option<String>,
) -> Result<()> {
    let schemas_dir_path = io::concat_path(&folder_path, SCHEMAS_DIR_NAME);
    let events_dir_path = io::concat_path(&folder_path, EVENTS_DIR_NAME);
    let migrations_dir_path = io::concat_path(&folder_path, MIGRATIONS_DIR_NAME);

    ensures_folder_exists(&schemas_dir_path)?;
    ensures_folder_exists(&events_dir_path)?;
    ensures_folder_exists(&migrations_dir_path)?;

    let now = chrono::Local::now();

    rename_migrations_files_to_match_current_date(now, &migrations_dir_path)?;
    rename_down_migrations_files_to_match_current_date(now, &migrations_dir_path)?;

    if traditional {
        // extract surql files
        let schema_definitions = io::extract_schema_definitions(config_file, None)?;
        let event_definitions = io::extract_event_definitions(config_file, None);

        // concat surql statements
        let schemas_statements = parse_statements(&schema_definitions)?;
        let events_statements = parse_statements(&event_definitions)?;

        let statements = schemas_statements
            .into_iter()
            .chain(events_statements)
            .collect::<Vec<_>>();

        create_initial_migration(&migrations_dir_path, &statements)?;
        create_initial_down_migration(&migrations_dir_path, &statements)?;

        // remove all files from schemas and events folders
        fs::remove_dir_all(schemas_dir_path)?;
        fs::remove_dir_all(events_dir_path)?;
    }

    Ok(())
}

const APPROXIMATE_LENGTH_PER_STATEMENT: usize = 50;
const INITIAL_MIGRATION_FILENAME: &str = "__Initial.surql";

fn create_initial_migration(
    migrations_dir_path: &Path,
    statements: &Vec<::surrealdb::sql::Statement>,
) -> Result<()> {
    let mut forward_content =
        String::with_capacity(APPROXIMATE_LENGTH_PER_STATEMENT * statements.len());
    let mut previous_idiom: Option<String> = None;

    for statement in statements {
        let idiom = match statement {
            ::surrealdb::sql::Statement::Define(define_statement) => match define_statement {
                DefineStatement::Table(DefineTableStatement { name, .. }) => Some(name.to_string()),
                DefineStatement::Field(DefineFieldStatement { what, .. }) => Some(what.to_string()),
                DefineStatement::Index(DefineIndexStatement { what, .. }) => Some(what.to_string()),
                DefineStatement::Event(DefineEventStatement { what, .. }) => Some(what.to_string()),
                _ => None,
            },
            _ => None,
        };

        if previous_idiom.is_some() && idiom != previous_idiom {
            forward_content.push('\n');
        }

        forward_content.push_str(&statement.to_string());
        forward_content.push_str(";\n");

        if idiom.is_some() {
            previous_idiom = idiom;
        }
    }

    let initial_file = migrations_dir_path.join(INITIAL_MIGRATION_FILENAME);
    fs::write(&initial_file, forward_content)?;

    Ok(())
}

fn create_initial_down_migration(
    migrations_dir_path: &Path,
    statements: &[::surrealdb::sql::Statement],
) -> Result<()> {
    let mut forward_content =
        String::with_capacity(APPROXIMATE_LENGTH_PER_STATEMENT * statements.len());
    let mut previous_idiom: Option<String> = None;

    for statement in statements.iter().rev() {
        let idiom = match statement {
            ::surrealdb::sql::Statement::Define(define_statement) => match define_statement {
                DefineStatement::Table(DefineTableStatement { name, .. }) => Some(name.to_string()),
                DefineStatement::Field(DefineFieldStatement { what, .. }) => Some(what.to_string()),
                DefineStatement::Index(DefineIndexStatement { what, .. }) => Some(what.to_string()),
                DefineStatement::Event(DefineEventStatement { what, .. }) => Some(what.to_string()),
                _ => None,
            },
            _ => None,
        };

        if previous_idiom.is_some() && idiom != previous_idiom {
            forward_content.push('\n');
        }

        let remove_statement = match statement {
            ::surrealdb::sql::Statement::Define(define_statement) => match define_statement {
                DefineStatement::Table(DefineTableStatement { name, .. }) => {
                    let mut s = RemoveTableStatement::default();
                    s.name = name.clone();

                    Ok(RemoveStatement::Table(s))
                }
                DefineStatement::Field(DefineFieldStatement { name, what, .. }) => {
                    let mut s = RemoveFieldStatement::default();
                    s.name = name.clone();
                    s.what = what.clone();

                    Ok(RemoveStatement::Field(s))
                }
                DefineStatement::Index(DefineIndexStatement { name, what, .. }) => {
                    let mut s = RemoveIndexStatement::default();
                    s.name = name.clone();
                    s.what = what.clone();

                    Ok(RemoveStatement::Index(s))
                }
                DefineStatement::Event(DefineEventStatement { name, what, .. }) => {
                    let mut s = RemoveEventStatement::default();
                    s.name = name.clone();
                    s.what = what.clone();

                    Ok(RemoveStatement::Event(s))
                }
                DefineStatement::Access(DefineAccessStatement { name, base, .. }) => {
                    let mut s = RemoveAccessStatement::default();
                    s.name = name.clone();
                    s.base = base.clone();

                    Ok(RemoveStatement::Access(s))
                }
                _ => Err(eyre!(
                    "DEFINE statements are not supported in remove statements"
                )),
            },
            _ => Err(eyre!(
                "DEFINE statements are not supported in remove statements"
            )),
        }?;

        forward_content.push_str(&remove_statement.to_string());
        forward_content.push_str(";\n");

        if idiom.is_some() {
            previous_idiom = idiom;
        }
    }

    let down_folder = migrations_dir_path.join(DOWN_MIGRATIONS_DIR_NAME);

    ensures_folder_exists(&down_folder)?;

    let initial_file = down_folder.join(INITIAL_MIGRATION_FILENAME);
    fs::write(&initial_file, forward_content)?;

    Ok(())
}

fn fails_if_folder_already_exists(dir_path: &Path, dir_name: &str) -> Result<()> {
    match dir_path.exists() {
        true => Err(eyre!("'{}' folder already exists.", dir_name)),
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

    let to = match folder_path {
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

fn rename_migrations_files_to_match_current_date(
    now: DateTime<Local>,
    migrations_dir_path: &PathBuf,
) -> Result<()> {
    let regex = regex::Regex::new(r"^YYYYMMDD_HHMM(\d{2})_")?;

    let migrations_dir = std::fs::read_dir(migrations_dir_path)?;

    let migration_filenames_to_rename = migrations_dir
        .filter_map(|entry| match entry {
            Ok(file) => {
                let file_name = file.file_name();
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
    migrations_dir_path: &Path,
) -> Result<()> {
    let down_migrations_dir_path = migrations_dir_path.join(DOWN_MIGRATIONS_DIR_NAME);

    if down_migrations_dir_path.exists() {
        rename_migrations_files_to_match_current_date(now, &down_migrations_dir_path)?;
    }

    Ok(())
}
