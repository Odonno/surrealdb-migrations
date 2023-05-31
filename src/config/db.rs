use anyhow::Result;

use super::common::{load_config, retrieve_config_value};

#[allow(dead_code)]
pub struct DbConfig {
    pub address: Option<String>,
    pub url: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub ns: Option<String>,
    pub db: Option<String>,
}

#[allow(dead_code)]
pub fn retrieve_db_config(config_file: Option<&str>) -> Result<DbConfig> {
    let config = load_config(config_file)?;

    let db_config = DbConfig {
        address: retrieve_config_value(&config, "db", "address"),
        url: retrieve_config_value(&config, "db", "url"),
        username: retrieve_config_value(&config, "db", "username"),
        password: retrieve_config_value(&config, "db", "password"),
        ns: retrieve_config_value(&config, "db", "ns"),
        db: retrieve_config_value(&config, "db", "db"),
    };

    Ok(db_config)
}
