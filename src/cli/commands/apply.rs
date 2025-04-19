use clap::Args;

#[derive(Args, Debug)]
pub struct ApplyArgs {
    /// Apply migrations up to this migration name.
    /// This parameter allows you to skip ulterior migrations.
    #[clap(long)]
    pub up: Option<String>,
    /// Apply migrations down to this migration name.
    /// This parameter allows you to rollback applied migrations.
    #[clap(long, num_args=0..=1, default_missing_value = "")]
    pub down: Option<String>,
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
