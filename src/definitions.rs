use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SchemaMigrationDefinition {
    pub schemas: String,
    pub events: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DefinitionDiff {
    pub schemas: Option<String>,
    pub events: Option<String>,
}
