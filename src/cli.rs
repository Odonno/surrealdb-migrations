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
        #[clap(short, long)]
        kind: ScaffoldKind,
    },
    /// Create a new migration file
    #[clap(aliases = vec!["c"])]
    Create {
        /// Name of the migration to create
        #[clap(short, long)]
        name: String,
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
