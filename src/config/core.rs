use super::common::retrieve_config_value;

pub enum TableSchemaDesign {
    Schemafull,
    Schemaless,
}

pub fn retrieve_folder_path() -> Option<String> {
    retrieve_config_value("core", "path")
}

pub fn retrieve_table_schema_design() -> Option<TableSchemaDesign> {
    let schema_str = retrieve_config_value("core", "schema");

    if let Some(schema_str) = schema_str {
        let schema_str = schema_str.to_lowercase();

        if schema_str == "full" {
            return Some(TableSchemaDesign::Schemafull);
        }
        if schema_str == "less" {
            return Some(TableSchemaDesign::Schemaless);
        }
    }

    return None;
}
