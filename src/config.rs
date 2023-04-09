use ini::Ini;
use std::path::Path;

pub struct DbConfig {
    pub url: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub ns: Option<String>,
    pub db: Option<String>,
}

fn load_config() -> Option<Ini> {
    let surrealdb_config_file = Path::new(".surrealdb");
    Ini::load_from_file(surrealdb_config_file).ok()
}

fn retrieve_config_value(section: &str, key: &str) -> Option<String> {
    let config = load_config()?;
    let section = config.section(Some(section))?;
    let value = section.get(key)?;

    Some(value.to_string())
}

pub fn retrieve_folder_path() -> Option<String> {
    retrieve_config_value("core", "path")
}

pub fn retrieve_db_config() -> DbConfig {
    DbConfig {
        url: retrieve_config_value("db", "url"),
        username: retrieve_config_value("db", "username"),
        password: retrieve_config_value("db", "password"),
        ns: retrieve_config_value("db", "ns"),
        db: retrieve_config_value("db", "db"),
    }
}
