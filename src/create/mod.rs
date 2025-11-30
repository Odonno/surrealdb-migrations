pub mod args;

pub use args::*;
use color_eyre::eyre::{eyre, Result};
use std::path::{Path, PathBuf};

use crate::{
    config,
    constants::{
        DOWN_MIGRATIONS_DIR_NAME, EVENTS_DIR_NAME, MIGRATIONS_DIR_NAME, SCHEMAS_DIR_NAME,
        SURQL_FILE_EXTENSION,
    },
    io,
    runbin::config::{retrieve_table_schema_design, TableSchemaDesign},
};

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
            ensures_folder_exists(&folder_path)?;
        }

        if file_path.exists() {
            return Err(eyre!("File {} already exists", filename));
        }
    }

    let content = generate_file_content(config_file, &operation, name)?;

    match dry_run {
        true => {
            println!("{content}");
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

            println!("File {filename} created successfully");
        }
    }

    Ok(())
}

fn get_filename(operation: &CreateOperation, name: &String) -> String {
    match operation {
        CreateOperation::Schema(_) => format!("{name}{SURQL_FILE_EXTENSION}"),
        CreateOperation::Event(_) => format!("{name}{SURQL_FILE_EXTENSION}"),
        CreateOperation::Migration(_) => {
            let now = chrono::Local::now();
            format!(
                "{}_{}_{}{}",
                now.format("%Y%m%d"),
                now.format("%H%M%S"),
                name,
                SURQL_FILE_EXTENSION
            )
        }
    }
}

fn generate_file_content(
    config_file: Option<&Path>,
    operation: &CreateOperation,
    name: String,
) -> Result<String> {
    let content = match operation {
        CreateOperation::Schema(args) => {
            let table_schema_design_str =
                get_table_schema_design_str(config_file, args.schemafull)?;
            let field_definitions = generate_field_definitions(&args.fields, name.to_string());

            format!(
                "DEFINE TABLE OVERWRITE {name} {table_schema_design_str};

{field_definitions}"
            )
        }
        CreateOperation::Event(args) => {
            let table_schema_design_str =
                get_table_schema_design_str(config_file, args.schemafull)?;
            let field_definitions = generate_field_definitions(&args.fields, name.to_string());

            format!(
                "DEFINE TABLE OVERWRITE {name} {table_schema_design_str};

{field_definitions}

DEFINE EVENT OVERWRITE {name} ON TABLE {name} WHEN $event == \"CREATE\" THEN (
    # TODO
);"
            )
        }
        CreateOperation::Migration(args) => args.content.to_owned().unwrap_or(String::new()),
    };

    Ok(content)
}

fn get_table_schema_design_str(
    config_file: Option<&Path>,
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
            TableSchemaDesign::Schemafull => SCHEMAFULL,
            TableSchemaDesign::Schemaless => SCHEMALESS,
        },
        None => SCHEMALESS,
    };
    Ok(value)
}

fn generate_field_definitions(fields: &Option<Vec<String>>, name: String) -> String {
    match fields {
        Some(fields) => fields
            .iter()
            .map(|field| format!("DEFINE FIELD OVERWRITE {field} ON {name};"))
            .collect::<Vec<String>>()
            .join("\n"),
        None => format!("# DEFINE FIELD OVERWRITE field ON {name};"),
    }
}

fn ensures_folder_exists(dir_path: &PathBuf) -> Result<()> {
    if !dir_path.exists() {
        fs_extra::dir::create_all(dir_path, false)?;
    }

    Ok(())
}
