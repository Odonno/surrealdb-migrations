use clap::{Parser, Subcommand};

#[derive(clap::ValueEnum, Debug, Clone)]
pub enum ScaffoldTemplate {
    Empty,
    Blog,
    Ecommerce,
}

#[derive(Parser, Debug)]
#[clap(name = "surrealdb-migrations", version, author = "Odonno")]
/// An awesome CLI for SurrealDB migrations (provides commands to scaffold, create and apply migrations).
pub struct Args {
    #[command(subcommand)]
    pub command: Action,
}

#[derive(Subcommand, Debug)]
pub enum Action {
    /// Scaffold a new SurrealDB project (with migrations)
    #[clap(aliases = vec!["s"])]
    Scaffold {
        /// Type of migration project to create
        template: ScaffoldTemplate,
    },
    /// Create a new migration file
    #[clap(aliases = vec!["c"])]
    Create {
        #[command(subcommand)]
        command: Option<CreateAction>,
        /// Name of the migration to create
        name: Option<String>,
    },
    /// Apply migration(s) to the database
    #[clap(aliases = vec!["a"])]
    Apply {
        /// Apply migrations up to this migration name.
        /// This parameter allows you to skip ulterior migrations.
        #[clap(long)]
        up: Option<String>,
        #[clap(long)]
        url: Option<String>,
        #[clap(long)]
        ns: Option<String>,
        #[clap(long)]
        db: Option<String>,
        #[clap(short, long)]
        username: Option<String>,
        #[clap(short, long)]
        password: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum CreateAction {
    #[clap(aliases = vec!["s"])]
    /// Generate a new schema file
    Schema {
        /// Name of the schema to generate
        name: String,
        /// A list of fields to define on the table
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
        /// A list of fields to define on the table
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
    },
}
