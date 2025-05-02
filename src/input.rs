#[cfg(feature = "branching")]
use crate::runbin::db_config::DbConfig;

/// The configuration used to connect to a SurrealDB instance.
#[derive(Default)]
pub struct SurrealdbConfiguration {
    /// Address of the surrealdb instance.
    /// Default value is `ws://localhost:8000`.
    pub address: Option<String>,
    /// Namespace to use inside the surrealdb instance.
    /// Default value is `test`.
    pub ns: Option<String>,
    /// Name of the database to use inside the surrealdb instance.
    /// Default value is `test`.
    pub db: Option<String>,
    /// Username used to authenticate to the surrealdb instance.
    /// Default value is `root`.
    pub username: Option<String>,
    /// Password used to authenticate to the surrealdb instance.
    /// Default value is `root`.
    pub password: Option<String>,
}

impl SurrealdbConfiguration {
    #[cfg(feature = "branching")]
    pub fn merge_with_config(&self, db_config: &DbConfig) -> Self {
        SurrealdbConfiguration {
            address: self.address.to_owned().or(db_config.address.to_owned()),
            username: self.username.to_owned().or(db_config.username.to_owned()),
            password: self.password.to_owned().or(db_config.password.to_owned()),
            ns: self.ns.to_owned().or(db_config.ns.to_owned()),
            db: self.db.to_owned().or(db_config.db.to_owned()),
        }
    }
}
