use clap::Subcommand;

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
