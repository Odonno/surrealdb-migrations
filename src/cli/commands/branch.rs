use clap::{Args, Subcommand};

#[derive(clap::ValueEnum, Debug, Clone)]
pub enum BranchMergeMode {
    SchemaOnly,
    All,
    Overwrite,
}

#[derive(Args, Debug)]
pub struct BranchArgs {
    #[command(subcommand)]
    pub command: Option<BranchAction>,
    /// Display information of the named branch
    pub name: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum BranchAction {
    #[clap(aliases = vec!["n"])]
    /// ** Preview ** Create a new branch
    New {
        /// Name of the branch to create (a random name will be generated if not provided)
        name: Option<String>,
        /// Address of the surrealdb instance.
        /// Default value is `ws://localhost:8000`.
        #[clap(long)]
        address: Option<String>,
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
    },
    #[clap(aliases = vec!["rm"])]
    /// ** Preview ** Remove an existing branch
    Remove {
        /// Name of the branch to remove
        name: String,
        /// Address of the surrealdb instance.
        /// Default value is `ws://localhost:8000`.
        #[clap(long)]
        address: Option<String>,
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
    },
    /// ** Preview ** Merge a branch and apply changes to the main branch
    Merge {
        /// Name of the branch to remove
        name: String,
        /// Mode to use when merging the branch
        /// - `schema-only`: only apply schema changes (including event changes)
        /// - `all`: apply schema and data changes
        /// - `overwrite`: overwrite the main branch with the branch to merge
        #[clap(long)]
        mode: BranchMergeMode,
        /// Address of the surrealdb instance.
        /// Default value is `ws://localhost:8000`.
        #[clap(long)]
        address: Option<String>,
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
    },
    /// ** Preview ** Display information of a branch
    Status {
        /// Name of a branch
        name: String,
    },
    #[clap(aliases = vec!["ls"])]
    /// ** Preview ** List all existing branches
    List {
        /// Address of the surrealdb instance.
        /// Default value is `ws://localhost:8000`.
        #[clap(long)]
        address: Option<String>,
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
    /// ** Preview ** Display the difference between the branch and its original branch
    Diff {
        /// Name of a branch
        name: String,
        /// Address of the surrealdb instance.
        /// Default value is `ws://localhost:8000`.
        #[clap(long)]
        address: Option<String>,
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
    },
}
