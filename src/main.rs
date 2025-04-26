use apply::ApplyArgs;
#[cfg(feature = "branching")]
use branch::{
    diff::BranchDiffArgs, list::ListBranchArgs, merge::MergeBranchArgs, new::NewBranchArgs,
    remove::RemoveBranchArgs, status::BranchStatusArgs,
};
use clap::Parser;
#[cfg(feature = "branching")]
use cli::BranchAction;
#[cfg(feature = "scaffold")]
use cli::ScaffoldAction;
use cli::{Action, Args, CreateAction};
use color_eyre::config::HookBuilder;
use color_eyre::config::Theme;
use color_eyre::eyre::eyre;
use color_eyre::eyre::Result;
use create::{CreateArgs, CreateEventArgs, CreateMigrationArgs, CreateOperation, CreateSchemaArgs};
use input::SurrealdbConfiguration;
use list::ListArgs;
use redo::RedoArgs;
#[cfg(feature = "scaffold-sql")]
use scaffold::schema::ScaffoldFromSchemaArgs;
#[cfg(feature = "scaffold")]
use scaffold::template::ScaffoldFromTemplateArgs;
use std::env;

use crate::surrealdb::create_surrealdb_client;

mod apply;
#[cfg(feature = "branching")]
mod branch;
mod cli;
mod common;
mod config;
mod constants;
mod create;
mod input;
mod io;
mod list;
mod models;
mod redo;
mod remove;
#[cfg(feature = "scaffold")]
mod scaffold;
mod surrealdb;
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
        Action::Scaffold { command } => match command {
            ScaffoldAction::Template {
                template,
                traditional,
            } => {
                let args = ScaffoldFromTemplateArgs {
                    template,
                    traditional,
                    config_file,
                };
                scaffold::template::main(args)
            }
            #[cfg(feature = "scaffold-sql")]
            ScaffoldAction::Schema {
                schema,
                db_type,
                preserve_casing,
                traditional,
            } => {
                let args = ScaffoldFromSchemaArgs {
                    schema,
                    db_type,
                    preserve_casing,
                    traditional,
                    config_file,
                };
                scaffold::schema::main(args)
            }
        },
        Action::Create(create_args) => {
            let cli::CreateArgs {
                command,
                name,
                down,
                content,
            } = create_args;

            match name {
                Some(name) => {
                    let operation =
                        CreateOperation::Migration(CreateMigrationArgs { down, content });
                    let args = CreateArgs {
                        name,
                        operation,
                        config_file,
                    };
                    create::main(args)
                }
                None => match command {
                    Some(CreateAction::Schema {
                        name,
                        fields,
                        dry_run,
                        schemafull,
                    }) => {
                        let operation = CreateOperation::Schema(CreateSchemaArgs {
                            fields,
                            dry_run,
                            schemafull,
                        });
                        let args = CreateArgs {
                            name,
                            operation,
                            config_file,
                        };
                        create::main(args)
                    }
                    Some(CreateAction::Event {
                        name,
                        fields,
                        dry_run,
                        schemafull,
                    }) => {
                        let operation = CreateOperation::Event(CreateEventArgs {
                            fields,
                            dry_run,
                            schemafull,
                        });
                        let args = CreateArgs {
                            name,
                            operation,
                            config_file,
                        };
                        create::main(args)
                    }
                    Some(CreateAction::Migration {
                        name,
                        down,
                        content,
                    }) => {
                        let operation =
                            CreateOperation::Migration(CreateMigrationArgs { down, content });
                        let args = CreateArgs {
                            name,
                            operation,
                            config_file,
                        };
                        create::main(args)
                    }
                    None => Err(eyre!("No action specified for `create` command")),
                },
            }
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
                validate_version_order,
                output,
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
                    validate_version_order,
                    config_file,
                    output,
                };
                redo::main(args).await
            } else {
                let operation = match (up, down, reset) {
                    (Some(up), None, false) if up.is_empty() => apply::ApplyOperation::UpSingle,
                    (Some(up), None, false) => apply::ApplyOperation::UpTo(up),
                    (None, Some(down), false) if down.is_empty() => {
                        apply::ApplyOperation::DownSingle
                    }
                    (None, Some(down), false) => apply::ApplyOperation::DownTo(down),
                    (None, None, true) => apply::ApplyOperation::Reset,
                    (None, None, false) => apply::ApplyOperation::Up,
                    _ => return Err(eyre!("Cannot apply when multiple args in conflict")),
                };
                let args = ApplyArgs {
                    operation,
                    db: &db,
                    dir: None,
                    display_logs: true,
                    dry_run,
                    validate_version_order,
                    config_file,
                    output,
                };
                apply::main(args).await
            }
        }
        Action::List(list_args) => {
            let cli::ListArgs {
                address,
                ns,
                db,
                username,
                password,
                no_color,
            } = list_args;

            let db_configuration = SurrealdbConfiguration {
                address,
                ns,
                db,
                username,
                password,
            };
            let args = ListArgs {
                db_configuration: &db_configuration,
                no_color,
                config_file,
            };
            list::main(args).await
        }
        #[cfg(feature = "branching")]
        Action::Branch(branch_args) => {
            let cli::BranchArgs { command, name } = branch_args;

            match name {
                Some(name) => {
                    let args = BranchStatusArgs { name, config_file };
                    branch::status::main(args).await
                }
                None => match command {
                    Some(BranchAction::New {
                        name,
                        address,
                        ns,
                        db,
                        username,
                        password,
                    }) => {
                        let db_configuration = SurrealdbConfiguration {
                            address,
                            ns,
                            db,
                            username,
                            password,
                        };
                        let args = NewBranchArgs {
                            name,
                            db_configuration: &db_configuration,
                            config_file,
                        };
                        branch::new::main(args).await
                    }
                    Some(BranchAction::Remove {
                        name,
                        address,
                        ns,
                        db,
                        username,
                        password,
                    }) => {
                        let db_configuration = SurrealdbConfiguration {
                            address,
                            ns,
                            db,
                            username,
                            password,
                        };
                        let args = RemoveBranchArgs {
                            name,
                            db_configuration: &db_configuration,
                            config_file,
                        };
                        branch::remove::main(args).await
                    }
                    Some(BranchAction::Merge {
                        name,
                        mode,
                        address,
                        ns,
                        db,
                        username,
                        password,
                    }) => {
                        let db_configuration = SurrealdbConfiguration {
                            address,
                            ns,
                            db,
                            username,
                            password,
                        };
                        let args = MergeBranchArgs {
                            name,
                            mode,
                            db_configuration: &db_configuration,
                            config_file,
                        };
                        branch::merge::main(args).await
                    }
                    Some(BranchAction::Status { name }) => {
                        let args = BranchStatusArgs { name, config_file };
                        branch::status::main(args).await
                    }
                    Some(BranchAction::List {
                        address,
                        ns,
                        db,
                        username,
                        password,
                        no_color,
                    }) => {
                        let db_configuration = SurrealdbConfiguration {
                            address,
                            ns,
                            db,
                            username,
                            password,
                        };
                        let args = ListBranchArgs {
                            db_configuration: &db_configuration,
                            no_color,
                            config_file,
                        };
                        branch::list::main(args).await
                    }
                    Some(BranchAction::Diff {
                        name,
                        address,
                        ns,
                        db,
                        username,
                        password,
                    }) => {
                        let db_configuration = SurrealdbConfiguration {
                            address,
                            ns,
                            db,
                            username,
                            password,
                        };
                        let args = BranchDiffArgs {
                            name,
                            db_configuration: &db_configuration,
                            config_file,
                        };
                        branch::diff::main(args).await
                    }
                    None => Err(eyre!("No action specified for `branch` command")),
                },
            }
        }
    }
}
