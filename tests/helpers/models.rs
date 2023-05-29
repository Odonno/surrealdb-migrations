use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Branch {
    pub name: String,
    pub from_ns: String,
    pub from_db: String,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Post {
    pub title: String,
    pub content: String,
}
