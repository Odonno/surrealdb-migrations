use anyhow::{anyhow, Result};
use std::path::PathBuf;

use crate::{
    config::{self, retrieve_table_schema_design},
    constants::{DOWN_MIGRATIONS_DIR_NAME, EVENTS_DIR_NAME, MIGRATIONS_DIR_NAME, SCHEMAS_DIR_NAME},
    io,
};

pub struct CreateArgs<'a> {
    pub name: String,
    pub operation: CreateOperation,
    pub config_file: Option<&'a str>,
}

pub enum CreateOperation {
    Schema(CreateSchemaArgs),
    Event(CreateEventArgs),
    Migration(CreateMigrationArgs),
}

pub struct CreateSchemaArgs {
    pub fields: Option<Vec<String>>,
    pub dry_run: bool,
    pub schemafull: bool,
}

pub struct CreateEventArgs {
    pub fields: Option<Vec<String>>,
    pub dry_run: bool,
    pub schemafull: bool,
}

pub struct CreateMigrationArgs {
    pub down: bool,
    pub content: Option<String>,
}

pub fn main(args: CreateArgs) -> Result<()> {
    let CreateArgs {
        name,
        operation,
        config_file,
    } = args;

    let dry_run = match &operation {
        CreateOperation::Schema(args) => args.dry_run,
        CreateOperation::Event(args) => args.dry_run,
        CreateOperation::Migration(_) => false,
    };

    let folder_path = config::retrieve_folder_path(config_file);

    let dir_name = match operation {
        CreateOperation::Schema(_) => SCHEMAS_DIR_NAME,
        CreateOperation::Event(_) => EVENTS_DIR_NAME,
        CreateOperation::Migration(_) => MIGRATIONS_DIR_NAME,
    };

    let folder_path = io::concat_path(&folder_path, dir_name);

    let filename = get_filename(&operation, &name);

    let file_path = folder_path.join(&filename);

    if !dry_run {
        if !folder_path.exists() {
            return Err(anyhow!("Directory {} doesn't exist", dir_name));
        }

        if file_path.exists() {
            return Err(anyhow!("File {} already exists", filename));
        }
    }

    let content = generate_file_content(config_file, &operation, name)?;

    match dry_run {
        true => {
            println!("{}", content);
        }
        false => {
            fs_extra::file::write_all(&file_path, &content)?;

            let should_create_down_file = match operation {
                CreateOperation::Migration(CreateMigrationArgs { down, .. }) => down,
                _ => false,
            };

            if should_create_down_file {
                let down_folder_path = folder_path.join(DOWN_MIGRATIONS_DIR_NAME);
                ensures_folder_exists(&down_folder_path)?;

                let down_file_path = down_folder_path.join(&filename);
                fs_extra::file::write_all(down_file_path, "")?;
            }

            println!("File {} created successfully", filename);
        }
    }

    Ok(())
}

fn get_filename(operation: &CreateOperation, name: &String) -> String {
    match operation {
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
    }
}

fn generate_file_content(
    config_file: Option<&str>,
    operation: &CreateOperation,
    name: String,
) -> Result<String> {
    let content = match operation {
        CreateOperation::Schema(args) => {
            let table_schema_design_str =
                get_table_schema_design_str(config_file, args.schemafull)?;
            let field_definitions = generate_field_definitions(&args.fields, name.to_string());

            format!(
                "DEFINE TABLE {0} {1};

{2}",
                name, table_schema_design_str, field_definitions
            )
        }
        CreateOperation::Event(args) => {
            let table_schema_design_str =
                get_table_schema_design_str(config_file, args.schemafull)?;
            let field_definitions = generate_field_definitions(&args.fields, name.to_string());

            format!(
                "DEFINE TABLE {0} {1};

{2}

DEFINE EVENT {0} ON TABLE {0} WHEN $before == NONE THEN (
    # TODO
);",
                name, table_schema_design_str, field_definitions
            )
        }
        CreateOperation::Migration(args) => args.content.to_owned().unwrap_or(String::new()),
    };

    Ok(content)
}

fn get_table_schema_design_str(
    config_file: Option<&str>,
    schemafull: bool,
) -> Result<&'static str> {
    const SCHEMAFULL: &str = "SCHEMAFULL";
    const SCHEMALESS: &str = "SCHEMALESS";

    if schemafull {
        return Ok(SCHEMAFULL);
    }

    let table_schema_design = retrieve_table_schema_design(config_file);

    let value = match table_schema_design {
        Some(table_schema_design) => match table_schema_design {
            config::TableSchemaDesign::Schemafull => SCHEMAFULL,
            config::TableSchemaDesign::Schemaless => SCHEMALESS,
        },
        None => SCHEMALESS,
    };
    Ok(value)
}

fn generate_field_definitions(fields: &Option<Vec<String>>, name: String) -> String {
    match fields {
        Some(fields) => fields
            .iter()
            .map(|field| format!("DEFINE FIELD {} ON {};", field, name))
            .collect::<Vec<String>>()
            .join("\n"),
        None => format!("# DEFINE FIELD field ON {};", name),
    }
}

fn ensures_folder_exists(dir_path: &PathBuf) -> Result<()> {
    if !dir_path.exists() {
        fs_extra::dir::create_all(dir_path, false)?;
    }

    Ok(())
}
