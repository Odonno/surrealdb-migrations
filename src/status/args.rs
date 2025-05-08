use std::path::Path;

use crate::{cli, input::SurrealdbConfiguration};

pub struct StatusArgs<'a> {
    pub db_configuration: SurrealdbConfiguration,
    pub no_color: bool,
    pub config_file: Option<&'a Path>,
}

impl<'a> StatusArgs<'a> {
    pub fn from(value: cli::StatusArgs, config_file: Option<&'a Path>) -> Self {
        let cli::StatusArgs {
            address,
            ns,
            db,
            username,
            password,
            no_color,
        } = value;

        let db_configuration = SurrealdbConfiguration {
            address,
            ns,
            db,
            username,
            password,
        };

        StatusArgs {
            db_configuration,
            no_color,
            config_file,
        }
    }
}
