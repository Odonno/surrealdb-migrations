use clap::Args;

#[derive(Args, Debug)]
pub struct ListArgs {
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
    pub no_color: bool,
}
