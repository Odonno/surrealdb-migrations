/// The configuration used to connect to a SurrealDB instance.
pub struct SurrealdbConfiguration {
    /// Address of the surrealdb instance.
    /// Default value is `ws://localhost:8000`.
    pub address: Option<String>,
    #[deprecated(since = "0.9.6", note = "Please use `address` instead")]
    /// Url of the surrealdb instance.
    /// Default value is `localhost:8000`.
    pub url: Option<String>,
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
    pub fn default() -> SurrealdbConfiguration {
        SurrealdbConfiguration {
            address: None,
            url: None,
            ns: None,
            db: None,
            username: None,
            password: None,
        }
    }
}
