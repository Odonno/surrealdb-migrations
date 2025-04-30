use std::{env, path::Path};

use crate::config::common::{load_config, retrieve_config_value};

use super::env::ENV_SCHEMA;

pub enum TableSchemaDesign {
    Schemafull,
    Schemaless,
}

pub fn retrieve_table_schema_design(config_file: Option<&Path>) -> Option<TableSchemaDesign> {
    let config = load_config(config_file);

    let schema_str = if let Some(config) = config {
        retrieve_config_value(&config, "core", "schema").or(env::var(ENV_SCHEMA).ok())
    } else {
        env::var(ENV_SCHEMA).ok()
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
