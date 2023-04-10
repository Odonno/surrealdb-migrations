use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ScriptMigration {
    pub script_name: String,
    pub executed_at: String,
}
