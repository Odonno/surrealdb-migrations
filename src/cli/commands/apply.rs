use clap::Args;

#[derive(Args, Debug)]
pub struct ApplyArgs {
    /// Apply migrations up to this migration name.
    /// This parameter allows you to skip ulterior migrations.
    ///
    /// Note: Apply a single migration when no value is provided.
    #[clap(long, num_args=0..=1, default_missing_value = "", conflicts_with_all = vec!["down", "reset", "redo"])]
    pub up: Option<String>,
    /// Apply migrations down to this migration name.
    /// This parameter allows you to rollback applied migrations.
    ///
    /// Note: Rollback a single migration when no value is provided.
    #[clap(long, num_args=0..=1, default_missing_value = "", conflicts_with_all = vec!["up", "reset", "redo"])]
    pub down: Option<String>,
    /// Resets the database, i.e. apply all migrations down.
    /// This parameter allows you to rollback ALL applied migrations.
    #[clap(long, conflicts_with_all = vec!["up", "down", "redo"])]
    pub reset: bool,
    /// Re-apply an already applied migration script.
    /// Please specify the name of the migration to re-apply.
    #[clap(long, conflicts_with_all = vec!["up", "down", "reset"])]
    pub redo: Option<String>,
    /// Address of the surrealdb instance.
    /// Default value is `ws://localhost:8000`.
    #[clap(long)]
    pub address: Option<String>,
    /// Namespace to use inside the surrealdb instance.
    /// Default value is `test`.
    #[clap(long)]
    pub ns: Option<String>,
    /// Name of the database to use inside the surrealdb instance.
    /// Default value is `test`.
    #[clap(long)]
    pub db: Option<String>,
    /// Username used to authenticate to the surrealdb instance.
    /// Default value is `root`.
    #[clap(short, long)]
    pub username: Option<String>,
    /// Password used to authenticate to the surrealdb instance.
    /// Default value is `root`.
    #[clap(short, long)]
    pub password: Option<String>,
    #[clap(long)]
    pub dry_run: bool,
    /// Validate the version order of the migrations so that you cannot run migrations if there are
    /// gaps in the migrations history.
    #[clap(long)]
    pub validate_version_order: bool,
    /// Output the surql statements to the console.
    #[clap(short, long, requires = "dry_run")]
    pub output: bool,
}
