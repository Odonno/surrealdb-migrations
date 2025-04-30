use std::{env, path::Path};

use crate::config::common::{load_config, retrieve_config_value};

use super::env::{
    ENV_SURREAL_ADDRESS, ENV_SURREAL_DB, ENV_SURREAL_NS, ENV_SURREAL_PASS, ENV_SURREAL_USER,
};

#[derive(Default)]
pub struct DbConfig {
    pub address: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub ns: Option<String>,
    pub db: Option<String>,
}

pub fn retrieve_db_config(config_file: Option<&Path>) -> DbConfig {
    let config = load_config(config_file);

    if let Some(config) = config {
        DbConfig {
            address: retrieve_config_value(&config, "db", "address")
                .or(env::var(ENV_SURREAL_ADDRESS).ok()),
            username: retrieve_config_value(&config, "db", "username")
                .or(env::var(ENV_SURREAL_USER).ok()),
            password: retrieve_config_value(&config, "db", "password")
                .or(env::var(ENV_SURREAL_PASS).ok()),
            ns: retrieve_config_value(&config, "db", "ns").or(env::var(ENV_SURREAL_NS).ok()),
            db: retrieve_config_value(&config, "db", "db").or(env::var(ENV_SURREAL_DB).ok()),
        }
    } else {
        DbConfig {
            address: env::var(ENV_SURREAL_ADDRESS).ok(),
            username: env::var(ENV_SURREAL_USER).ok(),
            password: env::var(ENV_SURREAL_PASS).ok(),
            ns: env::var(ENV_SURREAL_NS).ok(),
            db: env::var(ENV_SURREAL_DB).ok(),
        }
    }
}
