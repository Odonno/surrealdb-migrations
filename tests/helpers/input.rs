use super::DbInstance;

#[derive(Debug, Default, Clone)]
pub struct SurrealdbConfiguration {
    pub address: Option<String>,
    pub ns: Option<String>,
    pub db: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl SurrealdbConfiguration {
    pub fn from(db_instance: DbInstance, db_name: &str) -> Self {
        let username = match db_instance {
            DbInstance::Root => "root",
            DbInstance::Admin => "admin",
        };
        let password = match db_instance {
            DbInstance::Root => "root",
            DbInstance::Admin => "admin",
        };
        let port = match db_instance {
            DbInstance::Root => "8000",
            DbInstance::Admin => "8001",
        };

        SurrealdbConfiguration {
            address: Some(format!("ws://localhost:{port}")),
            username: Some(username.to_string()),
            password: Some(password.to_string()),
            ns: Some("test".to_string()),
            db: Some(db_name.to_string()),
        }
    }
}
