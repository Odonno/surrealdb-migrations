use ini::Ini;
use std::path::Path;

pub fn load_config(config_file: Option<&Path>) -> Option<Ini> {
    let ini = match config_file {
        Some(config_file) => Ini::load_from_file(config_file),
        None => Ini::load_from_file(".surrealdb"),
    };

    ini.ok()
}

pub fn retrieve_config_value(config: &Ini, section: &str, key: &str) -> Option<String> {
    let section = config.section(Some(section))?;
    let value = section.get(key)?;

    Some(value.to_string())
}
