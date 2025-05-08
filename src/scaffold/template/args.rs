use std::path::Path;

use crate::cli::ScaffoldTemplate;

pub struct ScaffoldFromTemplateArgs<'a> {
    pub template: ScaffoldTemplate,
    pub traditional: bool,
    pub config_file: Option<&'a Path>,
}
