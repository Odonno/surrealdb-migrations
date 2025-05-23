use serde::{Deserialize, Serialize};
use surrealdb::sql::Datetime;

#[derive(Serialize, Deserialize, Debug)]
pub struct ScriptMigration {
    pub script_name: String,
    pub executed_at: String,
    pub checksum: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Branch {
    pub name: String,
    pub from_ns: String,
    pub from_db: String,
    pub created_at: Datetime,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct SchemaMigrationDefinition {
    pub schemas: String,
    pub events: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DefinitionDiff {
    pub schemas: Option<String>,
    pub events: Option<String>,
}
