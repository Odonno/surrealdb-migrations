use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[cfg(feature = "branching")]
use super::BranchArgs;
#[cfg(feature = "scaffold")]
use super::ScaffoldAction;
use super::{ApplyArgs, CreateArgs, ListArgs, StatusArgs};

#[derive(Parser, Debug)]
#[clap(name = "surrealdb-migrations", version, author = "Odonno")]
/// An awesome CLI for SurrealDB migrations
/// (provides commands to scaffold, create and apply migrations).
pub struct Args {
    #[command(subcommand)]
    pub command: Action,
    /// Path to the configuration file
    /// Default value is `.surrealdb`.
    #[clap(long, global = true)]
    pub config_file: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Action {
    #[cfg(feature = "scaffold")]
    /// Scaffold a new SurrealDB project (with migrations)
    #[clap(aliases = vec!["s"])]
    Scaffold {
        #[command(subcommand)]
        command: ScaffoldAction,
    },
    /// Create a new migration file
    #[clap(aliases = vec!["c"])]
    Create(CreateArgs),
    /// Remove last migration file
    #[clap(aliases = vec!["rm"])]
    Remove,
    /// Apply migration(s) to the database
    #[clap(aliases = vec!["a"])]
    Apply(ApplyArgs),
    /// List all migrations applied to the database
    #[clap(aliases = vec!["ls"])]
    List(ListArgs),
    /// Get a glimpse at the status of the database migrations
    #[clap(aliases = vec!["st"])]
    Status(StatusArgs),
    #[cfg(feature = "branching")]
    /// ** Preview ** A set of commands for database branching
    #[clap(aliases = vec!["b"])]
    Branch(BranchArgs),
}
