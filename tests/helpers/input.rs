pub struct SurrealdbConfiguration {
    pub address: Option<String>,
    pub url: Option<String>,
    pub ns: Option<String>,
    pub db: Option<String>,
    pub username: Option<String>,
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
