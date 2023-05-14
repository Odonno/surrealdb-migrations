use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(name = "surrealdb-migrations", version, author = "Odonno")]
/// An awesome CLI for SurrealDB migrations
/// (provides commands to scaffold, create and apply migrations).
pub struct Args {
    #[command(subcommand)]
    pub command: Action,
}

#[derive(Subcommand, Debug)]
pub enum Action {
    /// Scaffold a new SurrealDB project (with migrations)
    #[clap(aliases = vec!["s"])]
    Scaffold {
        #[command(subcommand)]
        command: ScaffoldAction,
    },
    /// Create a new migration file
    #[clap(aliases = vec!["c"])]
    Create {
        #[command(subcommand)]
        command: Option<CreateAction>,
        /// Name of the migration to create
        name: Option<String>,
        /// Also generates a new DOWN migration file inside the `/migrations/down` folder
        #[clap(long)]
        down: bool,
    },
    /// Remove last migration file
    #[clap(aliases = vec!["rm"])]
    Remove {},
    /// Apply migration(s) to the database
    #[clap(aliases = vec!["a"])]
    Apply {
        /// Apply migrations up to this migration name.
        /// This parameter allows you to skip ulterior migrations.
        #[clap(long)]
        up: Option<String>,
        /// Url of the surrealdb instance.
        /// Default value is `localhost:8000`.
        #[clap(long)]
        url: Option<String>,
        /// Namespace to use inside the surrealdb instance.
        /// Default value is `test`.
        #[clap(long)]
        ns: Option<String>,
        /// Name of the database to use inside the surrealdb instance.
        /// Default value is `test`.
        #[clap(long)]
        db: Option<String>,
        /// Username used to authenticate to the surrealdb instance.
        /// Default value is `root`.
        #[clap(short, long)]
        username: Option<String>,
        /// Password used to authenticate to the surrealdb instance.
        /// Default value is `root`.
        #[clap(short, long)]
        password: Option<String>,
        #[clap(long)]
        dry_run: bool,
    },
    /// List all migrations applied to the database
    #[clap(aliases = vec!["ls"])]
    List {
        /// Url of the surrealdb instance.
        /// Default value is `localhost:8000`.
        #[clap(long)]
        url: Option<String>,
        /// Namespace to use inside the surrealdb instance.
        /// Default value is `test`.
        #[clap(long)]
        ns: Option<String>,
        /// Name of the database to use inside the surrealdb instance.
        /// Default value is `test`.
        #[clap(long)]
        db: Option<String>,
        /// Username used to authenticate to the surrealdb instance.
        /// Default value is `root`.
        #[clap(short, long)]
        username: Option<String>,
        /// Password used to authenticate to the surrealdb instance.
        /// Default value is `root`.
        #[clap(short, long)]
        password: Option<String>,
        #[clap(long)]
        no_color: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum ScaffoldAction {
    /// Scaffold a new project from a predefined template
    Template {
        /// Predefined template used to scaffold the project
        template: ScaffoldTemplate,
    },
    /// Scaffold a new project from an existing SQL schema file
    Schema {
        /// Path to the SQL schema file
        schema: String,
        /// Type of the database used in the SQL schema file
        #[clap(long)]
        db_type: ScaffoldSchemaDbType,
        /// Preserve casing of the table and column names instead of converting them to snake_case
        #[clap(long)]
        preserve_casing: bool,
    },
}

#[derive(clap::ValueEnum, Debug, Clone)]
pub enum ScaffoldTemplate {
    Empty,
    Blog,
    Ecommerce,
}

#[derive(clap::ValueEnum, Debug, Clone)]
#[clap(rename_all = "lower")]
pub enum ScaffoldSchemaDbType {
    BigQuery,
    ClickHouse,
    Hive,
    MsSql,
    MySql,
    PostgreSql,
    Redshift,
    SQLite,
    Snowflake,
}

#[derive(Subcommand, Debug)]
pub enum CreateAction {
    #[clap(aliases = vec!["s"])]
    /// Generate a new schema file
    Schema {
        /// Name of the schema to generate
        name: String,
        /// A list of fields to define on the table, using "," as a delimiter
        #[clap(short, long, value_delimiter = ',')]
        fields: Option<Vec<String>>,
        #[clap(long)]
        dry_run: bool,
    },
    #[clap(aliases = vec!["e"])]
    /// Generate a new event file
    Event {
        /// Name of the event to generate
        name: String,
        /// A list of fields to define on the table, using "," as a delimiter
        #[clap(short, long, value_delimiter = ',')]
        fields: Option<Vec<String>>,
        #[clap(long)]
        dry_run: bool,
    },
    #[clap(aliases = vec!["m"])]
    /// Generate a new migration file
    Migration {
        /// Name of the migration to generate
        name: String,
        /// Also generates a new DOWN migration file inside the `/migrations/down` folder
        #[clap(long)]
        down: bool,
    },
}
