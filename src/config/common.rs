use anyhow::Result;
use ini::Ini;
use std::path::Path;

pub fn load_config(config_file: Option<&str>) -> Result<Ini> {
    let config_file_path = config_file.unwrap_or(".surrealdb");
    let surrealdb_config_file = Path::new(&config_file_path);

    let ini = Ini::load_from_file(surrealdb_config_file)?;
    Ok(ini)
}

pub fn retrieve_config_value(config: &Ini, section: &str, key: &str) -> Option<String> {
    let section = config.section(Some(section))?;
    let value = section.get(key)?;

    Some(value.to_string())
}
