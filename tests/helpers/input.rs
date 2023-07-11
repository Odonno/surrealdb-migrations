#[derive(Debug, Default, Clone)]
pub struct SurrealdbConfiguration {
    pub address: Option<String>,
    pub url: Option<String>,
    pub ns: Option<String>,
    pub db: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}
