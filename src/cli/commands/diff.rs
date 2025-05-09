use clap::Args;

#[derive(Args, Debug)]
pub struct DiffArgs {
    #[clap(long)]
    pub no_color: bool,
}
