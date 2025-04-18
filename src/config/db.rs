use std::{env, path::Path};

use crate::constants;

use super::common::{load_config, retrieve_config_value};

#[allow(dead_code)]
#[derive(Default)]
pub struct DbConfig {
    pub address: Option<String>,
    pub url: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub ns: Option<String>,
    pub db: Option<String>,
}

#[allow(dead_code)]
pub fn retrieve_db_config(config_file: Option<&Path>) -> DbConfig {
    let config = load_config(config_file);

    if let Some(config) = config {
        DbConfig {
            address: retrieve_config_value(&config, "db", "address")
                .or(env::var(constants::ENV_SURREAL_ADDRESS).ok()),
            url: retrieve_config_value(&config, "db", "url"),
            username: retrieve_config_value(&config, "db", "username")
                .or(env::var(constants::ENV_SURREAL_USER).ok()),
            password: retrieve_config_value(&config, "db", "password")
                .or(env::var(constants::ENV_SURREAL_PASS).ok()),
            ns: retrieve_config_value(&config, "db", "ns")
                .or(env::var(constants::ENV_SURREAL_NS).ok()),
            db: retrieve_config_value(&config, "db", "db")
                .or(env::var(constants::ENV_SURREAL_DB).ok()),
        }
    } else {
        DbConfig {
            address: env::var(constants::ENV_SURREAL_ADDRESS).ok(),
            username: env::var(constants::ENV_SURREAL_USER).ok(),
            password: env::var(constants::ENV_SURREAL_PASS).ok(),
            ns: env::var(constants::ENV_SURREAL_NS).ok(),
            db: env::var(constants::ENV_SURREAL_DB).ok(),
            ..DbConfig::default()
        }
    }
}
