use apply::ApplyArgs;
#[cfg(feature = "branching")]
use branch::args::BranchArgs;
use clap::Parser;
use cli::{Action, Args};
use color_eyre::config::HookBuilder;
use color_eyre::config::Theme;
use color_eyre::eyre::Result;
use create::CreateArgs;
use diff::DiffArgs;
use input::SurrealdbConfiguration;
use list::ListArgs;
use models::ApplyOperation;
use redo::RedoArgs;
use runbin::surrealdb::create_surrealdb_client;
#[cfg(feature = "scaffold")]
use scaffold::args::ScaffoldArgs;
use status::StatusArgs;
use std::collections::HashSet;
use std::env;

mod apply;
#[cfg(feature = "branching")]
mod branch;
mod cli;
mod common;
mod config;
mod constants;
mod create;
mod diff;
mod file;
mod input;
mod io;
mod list;
mod models;
mod redo;
mod remove;
mod runbin;
#[cfg(feature = "scaffold")]
mod scaffold;
mod status;
mod surrealdb;
mod tags;
mod validate_checksum;
mod validate_version_order;

#[cfg(target_arch = "wasm32")]
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    sub_main().await
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<()> {
    sub_main().await
}

async fn sub_main() -> Result<()> {
    if env::var("NO_COLOR").unwrap_or(String::from("0")) == "1" {
        HookBuilder::default()
            .theme(Theme::new()) // disable colors
            .install()?;
    } else {
        color_eyre::install()?;
    }

    let args = Args::parse();

    let config_file = args.config_file.as_deref();

    match args.command {
        #[cfg(feature = "scaffold")]
        Action::Scaffold { command } => match ScaffoldArgs::from(command, config_file) {
            #[cfg(feature = "scaffold-sql")]
            ScaffoldArgs::Schema(args) => scaffold::schema::main(args),
            ScaffoldArgs::Template(args) => scaffold::template::main(args),
        },
        Action::Create(create_args) => {
            let args = CreateArgs::try_from(create_args, config_file)?;
            create::main(args)
        }
        Action::Remove => remove::main(config_file),
        Action::Apply(apply_args) => {
            let cli::ApplyArgs {
                up,
                down,
                reset,
                redo,
                address,
                ns,
                db,
                username,
                password,
                dry_run,
                validate_checksum,
                validate_version_order,
                output,
                tags,
            } = apply_args;

            let db_configuration = SurrealdbConfiguration {
                address,
                ns,
                db,
                username,
                password,
            };
            let db = create_surrealdb_client(config_file, &db_configuration).await?;

            if let Some(redo) = redo {
                let args = RedoArgs {
                    migration_script: redo,
                    db: &db,
                    dir: None,
                    display_logs: true,
                    dry_run,
                    validate_checksum,
                    validate_version_order,
                    config_file,
                    output,
                };
                redo::main(args).await
            } else {
                let operation = ApplyOperation::try_from(up, down, reset)?;
                let tags = tags.map(HashSet::from_iter);

                let args = ApplyArgs {
                    operation,
                    db: &db,
                    dir: None,
                    display_logs: true,
                    dry_run,
                    validate_checksum,
                    validate_version_order,
                    config_file,
                    output,
                    tags,
                };
                apply::main(args).await
            }
        }
        Action::List(list_args) => list::main(ListArgs::from(list_args, config_file)).await,
        Action::Status(status_args) => {
            status::main(StatusArgs::from(status_args, config_file)).await
        }
        #[cfg(feature = "branching")]
        Action::Branch(branch_args) => {
            let args = BranchArgs::try_from(branch_args, config_file)?;
            match args {
                BranchArgs::Diff(args) => branch::diff::main(args).await,
                BranchArgs::Merge(args) => branch::merge::main(args).await,
                BranchArgs::New(args) => branch::new::main(args).await,
                BranchArgs::List(args) => branch::list::main(args).await,
                BranchArgs::Remove(args) => branch::remove::main(args).await,
                BranchArgs::Status(args) => branch::status::main(args).await,
            }
        }
        Action::Diff(diff_args) => diff::main(DiffArgs::from(diff_args, config_file)).await,
    }
}
