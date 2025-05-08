use std::path::Path;

use crate::cli::ScaffoldAction;

#[cfg(feature = "scaffold-sql")]
use super::schema::ScaffoldFromSchemaArgs;
use super::template::ScaffoldFromTemplateArgs;

pub enum ScaffoldArgs<'a> {
    #[cfg(feature = "scaffold-sql")]
    Schema(ScaffoldFromSchemaArgs<'a>),
    Template(ScaffoldFromTemplateArgs<'a>),
}

impl<'a> ScaffoldArgs<'a> {
    pub fn from(value: ScaffoldAction, config_file: Option<&'a Path>) -> Self {
        match value {
            ScaffoldAction::Template {
                template,
                traditional,
            } => ScaffoldArgs::Template(ScaffoldFromTemplateArgs {
                template,
                traditional,
                config_file,
            }),
            #[cfg(feature = "scaffold-sql")]
            ScaffoldAction::Schema {
                schema,
                db_type,
                preserve_casing,
                traditional,
            } => ScaffoldArgs::Schema(ScaffoldFromSchemaArgs {
                schema,
                db_type,
                preserve_casing,
                traditional,
                config_file,
            }),
        }
    }
}
