use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ScriptMigration {
    pub script_name: String,
    pub executed_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Branch {
    pub name: String,
    pub from_ns: String,
    pub from_db: String,
    pub created_at: String,
}

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
