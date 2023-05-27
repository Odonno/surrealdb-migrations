use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct CreateArgs {
    #[command(subcommand)]
    pub command: Option<CreateAction>,
    /// Name of the migration to create
    pub name: Option<String>,
    /// Also generates a new DOWN migration file inside the `/migrations/down` folder
    #[clap(long)]
    pub down: bool,
    /// Content of the migration file
    #[clap(long)]
    pub content: Option<String>,
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
        /// Generate a `SCHEMAFULL` table
        #[clap(long)]
        schemafull: bool,
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
        /// Generate a `SCHEMAFULL` event table
        #[clap(long)]
        schemafull: bool,
    },
    #[clap(aliases = vec!["m"])]
    /// Generate a new migration file
    Migration {
        /// Name of the migration to generate
        name: String,
        /// Also generates a new DOWN migration file inside the `/migrations/down` folder
        #[clap(long)]
        down: bool,
        /// Content of the migration file
        #[clap(long)]
        content: Option<String>,
    },
}
