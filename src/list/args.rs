use std::path::Path;

use crate::{cli, input::SurrealdbConfiguration};

pub struct ListArgs<'a> {
    pub db_configuration: SurrealdbConfiguration,
    pub no_color: bool,
    pub config_file: Option<&'a Path>,
}

impl<'a> ListArgs<'a> {
    pub fn from(value: cli::ListArgs, config_file: Option<&'a Path>) -> Self {
        let cli::ListArgs {
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

        ListArgs {
            db_configuration,
            no_color,
            config_file,
        }
    }
}
