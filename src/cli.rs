use clap::{Parser, Subcommand};

#[derive(clap::ValueEnum, Debug, Clone)]
pub enum ScaffoldKind {
    Empty,
    Blog,
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
        /// Kind of migration project to create
        kind: ScaffoldKind,
    },
    /// Create a new migration file
    #[clap(aliases = vec!["c"])]
    Create {
        #[command(subcommand)]
        command: Option<CreateAction>,
        /// Name of the migration to create
        name: Option<String>,
    },
    /// Update migration(s) definitions (based on schemas and migrations created)
    #[clap(aliases = vec!["u"])]
    Update,
    /// Apply migration(s) to the database
    #[clap(aliases = vec!["a"])]
    Apply {
        url: Option<String>,
        ns: Option<String>,
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
    },
    #[clap(aliases = vec!["e"])]
    /// Generate a new event file
    Event {
        /// Name of the event to generate
        name: String,
    },
    #[clap(aliases = vec!["m"])]
    /// Generate a new migration file
    Migration {
        /// Name of the migration to generate
        name: String,
    },
}
