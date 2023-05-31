use anyhow::Result;

use super::common::{load_config, retrieve_config_value};

#[allow(dead_code)]
pub enum TableSchemaDesign {
    Schemafull,
    Schemaless,
}

pub fn retrieve_folder_path(config_file: Option<&str>) -> Result<Option<String>> {
    let config = load_config(config_file)?;
    let value = retrieve_config_value(&config, "core", "path");

    Ok(value)
}

#[allow(dead_code)]
pub fn retrieve_table_schema_design(
    config_file: Option<&str>,
) -> Result<Option<TableSchemaDesign>> {
    let config = load_config(config_file)?;
    let schema_str = retrieve_config_value(&config, "core", "schema");

    match schema_str {
        Some(schema_str) => {
            let value = parse_table_schema_design(schema_str);
            Ok(value)
        }
        _ => Ok(None),
    }
}

fn parse_table_schema_design(schema_str: String) -> Option<TableSchemaDesign> {
    match schema_str.to_lowercase().as_str() {
        "full" => Some(TableSchemaDesign::Schemafull),
        "less" => Some(TableSchemaDesign::Schemaless),
        _ => None,
    }
}
