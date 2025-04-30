use std::{env, path::Path};

use crate::constants;

use super::common::{load_config, retrieve_config_value};

pub fn retrieve_folder_path(config_file: Option<&Path>) -> Option<String> {
    let config = load_config(config_file);

    if let Some(config) = config {
        retrieve_config_value(&config, "core", "path").or(env::var(constants::ENV_PATH).ok())
    } else {
        env::var(constants::ENV_PATH).ok()
    }
}
