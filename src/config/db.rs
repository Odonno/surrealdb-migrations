use super::common::retrieve_config_value;

#[allow(dead_code)]
pub struct DbConfig {
    pub address: Option<String>,
    pub url: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub ns: Option<String>,
    pub db: Option<String>,
}

#[allow(dead_code)]
pub fn retrieve_db_config() -> DbConfig {
    DbConfig {
        address: retrieve_config_value("db", "address"),
        url: retrieve_config_value("db", "url"),
        username: retrieve_config_value("db", "username"),
        password: retrieve_config_value("db", "password"),
        ns: retrieve_config_value("db", "ns"),
        db: retrieve_config_value("db", "db"),
    }
}
