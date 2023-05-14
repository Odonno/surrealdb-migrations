use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

use crate::{
    config,
    constants::{EVENTS_DIR_NAME, MIGRATIONS_DIR_NAME, SCHEMAS_DIR_NAME},
};

pub struct CreateArgs {
    pub name: String,
    pub operation: CreateOperation,
}

pub enum CreateOperation {
    Schema(CreateSchemaArgs),
    Event(CreateEventArgs),
    Migration(CreateMigrationArgs),
}

pub struct CreateSchemaArgs {
    pub fields: Option<Vec<String>>,
    pub dry_run: bool,
}

pub struct CreateEventArgs {
    pub fields: Option<Vec<String>>,
    pub dry_run: bool,
}

pub struct CreateMigrationArgs {
    pub down: bool,
}

pub fn main(args: CreateArgs) -> Result<()> {
    let CreateArgs { name, operation } = args;

    let folder_path = config::retrieve_folder_path();

    let dir_name = match operation {
        CreateOperation::Schema(_) => SCHEMAS_DIR_NAME,
        CreateOperation::Event(_) => EVENTS_DIR_NAME,
        CreateOperation::Migration(_) => MIGRATIONS_DIR_NAME,
    };

    // Retrieve folder path
    let folder_path = match folder_path.to_owned() {
        Some(folder_path) => {
            let path = Path::new(&folder_path);
            path.join(dir_name)
        }
        None => Path::new(dir_name).to_path_buf(),
    };

    let filename = match &operation {
        CreateOperation::Schema(_) => format!("{}.surql", name),
        CreateOperation::Event(_) => format!("{}.surql", name),
        CreateOperation::Migration(_) => {
            let now = chrono::Local::now();
            format!(
                "{}_{}_{}.surql",
                now.format("%Y%m%d"),
                now.format("%H%M%S"),
                name
            )
        }
    };

    let file_path = folder_path.join(&filename);

    let dry_run = match &operation {
        CreateOperation::Schema(args) => args.dry_run,
        CreateOperation::Event(args) => args.dry_run,
        CreateOperation::Migration(_) => false,
    };

    if !dry_run {
        // Check that directory exists
        if !folder_path.exists() {
            return Err(anyhow!("Directory {} doesn't exist", dir_name));
        }

        // Check that file doesn't already exist
        if file_path.exists() {
            return Err(anyhow!("File {} already exists", filename));
        }
    }

    let content = match &operation {
        CreateOperation::Schema(args) => {
            // Generate field definitions
            let field_definitions = match &args.fields {
                Some(fields) => fields
                    .iter()
                    .map(|field| format!("DEFINE FIELD {} ON {};", field, name))
                    .collect::<Vec<String>>()
                    .join("\n"),
                None => format!("# DEFINE FIELD field ON {};", name),
            };

            format!(
                "DEFINE TABLE {0} SCHEMALESS;

{1}",
                name, field_definitions
            )
        }
        CreateOperation::Event(args) => {
            // Generate field definitions
            let field_definitions = match &args.fields {
                Some(fields) => fields
                    .iter()
                    .map(|field| format!("DEFINE FIELD {} ON {};", field, name))
                    .collect::<Vec<String>>()
                    .join("\n"),
                None => format!("# DEFINE FIELD field ON {};", name),
            };

            format!(
                "DEFINE TABLE {0} SCHEMALESS;

{1}

DEFINE EVENT {0} ON TABLE {0} WHEN $before == NONE THEN (
    # TODO
);",
                name, field_definitions
            )
        }
        CreateOperation::Migration(_) => "".to_string(),
    };

    match dry_run {
        true => {
            println!("{}", content);
        }
        false => {
            fs_extra::file::write_all(&file_path, &content)?;

            let should_create_down_file = match operation {
                CreateOperation::Migration(CreateMigrationArgs { down }) => down,
                _ => false,
            };

            if should_create_down_file {
                let down_folder_path = folder_path.join("down");
                ensures_folder_exists(&down_folder_path)?;

                let down_file_path = down_folder_path.join(&filename);
                fs_extra::file::write_all(&down_file_path, &content)?;
            }

            println!("File {} created successfully", filename);
        }
    }

    Ok(())
}

fn ensures_folder_exists(dir_path: &PathBuf) -> Result<()> {
    if !dir_path.exists() {
        fs_extra::dir::create_all(&dir_path, false)?;
    }

    Ok(())
}
