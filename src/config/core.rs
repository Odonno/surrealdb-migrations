use std::{env, path::Path};

use crate::constants;

use super::common::{load_config, retrieve_config_value};

#[allow(dead_code)]
pub enum TableSchemaDesign {
    Schemafull,
    Schemaless,
}

pub fn retrieve_folder_path(config_file: Option<&Path>) -> Option<String> {
    let config = load_config(config_file);

    if let Some(config) = config {
        retrieve_config_value(&config, "core", "path").or(env::var(constants::ENV_PATH).ok())
    } else {
        env::var(constants::ENV_PATH).ok()
    }
}

#[allow(dead_code)]
pub fn retrieve_table_schema_design(config_file: Option<&Path>) -> Option<TableSchemaDesign> {
    let config = load_config(config_file);

    let schema_str = if let Some(config) = config {
        retrieve_config_value(&config, "core", "schema").or(env::var(constants::ENV_SCHEMA).ok())
    } else {
        env::var(constants::ENV_SCHEMA).ok()
    };

    match schema_str {
        Some(schema_str) => parse_table_schema_design(schema_str),
        _ => None,
    }
}

fn parse_table_schema_design(schema_str: String) -> Option<TableSchemaDesign> {
    match schema_str.to_lowercase().as_str() {
        "full" => Some(TableSchemaDesign::Schemafull),
        "less" => Some(TableSchemaDesign::Schemaless),
        _ => None,
    }
}
