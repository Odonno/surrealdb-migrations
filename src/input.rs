/// The configuration used to connect to a SurrealDB instance.
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
    /// Create a new configuration with default values.
    #[allow(dead_code)]
    pub fn default() -> Self {
        SurrealdbConfiguration {
            address: None,
            ns: None,
            db: None,
            username: None,
            password: None,
        }
    }
}
