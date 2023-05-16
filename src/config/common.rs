use ini::Ini;
use std::path::Path;

pub fn load_config() -> Option<Ini> {
    let surrealdb_config_file = Path::new(".surrealdb");
    Ini::load_from_file(surrealdb_config_file).ok()
}

pub fn retrieve_config_value(section: &str, key: &str) -> Option<String> {
    let config = load_config()?;
    let section = config.section(Some(section))?;
    let value = section.get(key)?;

    Some(value.to_string())
}
