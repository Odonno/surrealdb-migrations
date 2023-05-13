/// The configuration used to connect to a SurrealDB instance.
pub struct SurrealdbConfiguration {
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
