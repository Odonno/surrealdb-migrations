use serde::{Deserialize, Serialize};
use surrealdb::types::{Datetime, SurrealValue};

#[derive(Debug, SurrealValue)]
pub struct ScriptMigration {
    pub script_name: String,
    pub executed_at: String,
    pub checksum: Option<String>,
}

#[derive(Debug, SurrealValue)]
pub struct Branch {
    pub name: String,
    pub from_ns: String,
    pub from_db: String,
    pub created_at: Datetime,
}

#[derive(Debug, Default, Serialize, Deserialize, SurrealValue)]
pub struct SchemaMigrationDefinition {
    pub schemas: String,
    pub events: String,
}

#[derive(Debug, Serialize, Deserialize, SurrealValue)]
pub struct DefinitionDiff {
    pub schemas: Option<String>,
    pub events: Option<String>,
}
