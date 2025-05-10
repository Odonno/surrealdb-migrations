use std::{collections::HashSet, env, path::Path};

use crate::config::common::{load_config, retrieve_config_value};

use super::env::{ENV_EXCLUDE_TAGS, ENV_SCHEMA, ENV_TAGS};

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

pub fn retrieve_tags(config_file: Option<&Path>) -> Option<HashSet<String>> {
    let config = load_config(config_file);

    config
        .and_then(|config| retrieve_config_value(&config, "filters", "tags"))
        .or(env::var(ENV_TAGS).ok())
        .map(|s| parse_tags(&s))
}

pub fn retrieve_exclude_tags(config_file: Option<&Path>) -> Option<HashSet<String>> {
    let config = load_config(config_file);

    config
        .and_then(|config| retrieve_config_value(&config, "filters", "exclude_tags"))
        .or(env::var(ENV_EXCLUDE_TAGS).ok())
        .map(|s| parse_tags(&s))
}

fn parse_tags(str: &str) -> HashSet<String> {
    HashSet::from_iter(str.split(',').map(|t| t.to_string()))
}
