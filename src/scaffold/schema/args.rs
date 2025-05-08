use std::path::Path;

use crate::cli::ScaffoldSchemaDbType;

pub struct ScaffoldFromSchemaArgs<'a> {
    pub schema: String,
    pub db_type: ScaffoldSchemaDbType,
    pub preserve_casing: bool,
    pub traditional: bool,
    pub config_file: Option<&'a Path>,
}
