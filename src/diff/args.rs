use std::path::Path;

use crate::cli;

pub struct DiffArgs<'a> {
    pub no_color: bool,
    pub config_file: Option<&'a Path>,
}

impl<'a> DiffArgs<'a> {
    pub fn from(value: cli::DiffArgs, config_file: Option<&'a Path>) -> Self {
        let cli::DiffArgs { no_color } = value;

        DiffArgs {
            no_color,
            config_file,
        }
    }
}
