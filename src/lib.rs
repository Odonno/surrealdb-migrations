use anyhow::Result;
use models::ScriptMigration;
mod apply;
mod config;
mod constants;
mod definitions;
mod list;
mod models;
mod surrealdb;

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

impl SurrealdbConfiguration {
    /// Create an instance of SurrealdbConfiguration with default values.
    ///
    /// ## Examples
    ///
    /// ```
    /// use surrealdb_migrations::SurrealdbConfiguration;
    ///
    /// let db_configuration = SurrealdbConfiguration::default();
    /// ```
    pub fn default() -> SurrealdbConfiguration {
        SurrealdbConfiguration {
            url: None,
            ns: None,
            db: None,
            username: None,
            password: None,
        }
    }
}

pub struct SurrealdbMigrations {
    db_configuration: SurrealdbConfiguration,
}

impl SurrealdbMigrations {
    /// Create a new instance of SurrealdbMigrations.
    pub fn new(db_configuration: SurrealdbConfiguration) -> SurrealdbMigrations {
        SurrealdbMigrations { db_configuration }
    }

    /// Apply schema definitions and apply all migrations.
    ///
    /// ## Examples
    ///
    /// ```rust,no_run
    /// use surrealdb_migrations::{SurrealdbConfiguration, SurrealdbMigrations};
    ///
    /// # tokio_test::block_on(async {
    /// let db_configuration = SurrealdbConfiguration::default();
    ///
    /// SurrealdbMigrations::new(db_configuration)
    ///     .up()
    ///     .await
    ///     .expect("Failed to apply migrations");
    /// # });
    /// ```
    pub async fn up(&self) -> Result<()> {
        apply::execute(
            None,
            self.db_configuration.url.clone(),
            self.db_configuration.ns.clone(),
            self.db_configuration.db.clone(),
            self.db_configuration.username.clone(),
            self.db_configuration.password.clone(),
            false,
        )
        .await
    }

    /// Apply schema definitions and all migrations up to and including the named migration.
    ///
    /// ## Arguments
    ///
    /// * `name` - This parameter allows you to skip ulterior migrations.
    ///
    /// ## Examples
    ///
    /// ```rust,no_run
    /// use surrealdb_migrations::{SurrealdbConfiguration, SurrealdbMigrations};
    ///
    /// # tokio_test::block_on(async {
    /// let db_configuration = SurrealdbConfiguration::default();
    ///
    /// SurrealdbMigrations::new(db_configuration)
    ///     .up_to("20230101_120002_AddPost")
    ///     .await
    ///     .expect("Failed to apply migrations");
    /// # });
    /// ```
    pub async fn up_to(&self, name: &str) -> Result<()> {
        apply::execute(
            Some(name.to_string()),
            self.db_configuration.url.clone(),
            self.db_configuration.ns.clone(),
            self.db_configuration.db.clone(),
            self.db_configuration.username.clone(),
            self.db_configuration.password.clone(),
            false,
        )
        .await
    }

    /// List script migrations that have been applied to the database.
    ///
    /// ## Examples
    ///
    /// ```rust,no_run
    /// use surrealdb_migrations::{SurrealdbConfiguration, SurrealdbMigrations};
    ///
    /// # tokio_test::block_on(async {
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db_configuration = SurrealdbConfiguration::default();
    ///
    /// let migrations_applied = SurrealdbMigrations::new(db_configuration)
    ///     .list()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// # main().await.unwrap();
    /// # });
    /// ```
    pub async fn list(&self) -> Result<Vec<ScriptMigration>> {
        let client = surrealdb::create_surrealdb_client(
            self.db_configuration.url.clone(),
            self.db_configuration.ns.clone(),
            self.db_configuration.db.clone(),
            self.db_configuration.username.clone(),
            self.db_configuration.password.clone(),
        )
        .await?;

        surrealdb::list_script_migration_ordered_by_execution_date(&client).await
    }
}
