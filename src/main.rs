use crate::surrealdb::create_surrealdb_client;

use anyhow::{anyhow, Result};
use apply::ApplyArgs;
use clap::Parser;
use cli::{Action, Args, BranchAction, CreateAction, ScaffoldAction};
use create::{CreateArgs, CreateEventArgs, CreateMigrationArgs, CreateOperation, CreateSchemaArgs};
use input::SurrealdbConfiguration;

mod apply;
mod branch;
mod cli;
mod config;
mod constants;
mod create;
mod definitions;
mod input;
mod io;
mod list;
mod models;
mod remove;
mod scaffold;
mod surrealdb;
mod validate_version_order;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Action::Scaffold { command } => match command {
            ScaffoldAction::Template { template } => scaffold::template::main(template),
            ScaffoldAction::Schema {
                schema,
                db_type,
                preserve_casing,
            } => scaffold::schema::main(schema, db_type, preserve_casing),
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
                    let args = CreateArgs { name, operation };
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
                        let args = CreateArgs { name, operation };
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
                        let args = CreateArgs { name, operation };
                        create::main(args)
                    }
                    Some(CreateAction::Migration {
                        name,
                        down,
                        content,
                    }) => {
                        let operation =
                            CreateOperation::Migration(CreateMigrationArgs { down, content });
                        let args = CreateArgs { name, operation };
                        create::main(args)
                    }
                    None => Err(anyhow!("No action specified for `create` command")),
                },
            }
        }
        Action::Remove {} => remove::main(),
        #[allow(deprecated)]
        Action::Apply(apply_args) => {
            let cli::ApplyArgs {
                up,
                down,
                address,
                url,
                ns,
                db,
                username,
                password,
                dry_run,
                validate_version_order,
            } = apply_args;

            let operation = match (up, down) {
                (Some(_), Some(_)) => {
                    return Err(anyhow!(
                        "You can't specify both `up` and `down` parameters at the same time"
                    ))
                }
                (Some(up), None) => apply::ApplyOperation::UpTo(up),
                (None, Some(down)) => apply::ApplyOperation::Down(down),
                (None, None) => apply::ApplyOperation::Up,
            };
            let db_configuration = SurrealdbConfiguration {
                address,
                url,
                ns,
                db,
                username,
                password,
            };
            let db = create_surrealdb_client(&db_configuration).await?;
            let args = ApplyArgs {
                operation,
                db: &db,
                dir: None,
                display_logs: true,
                dry_run,
                validate_version_order,
            };
            apply::main(args).await
        }
        #[allow(deprecated)]
        Action::List(list_args) => {
            let cli::ListArgs {
                address,
                url,
                ns,
                db,
                username,
                password,
                no_color,
            } = list_args;

            let db_configuration = SurrealdbConfiguration {
                address,
                url,
                ns,
                db,
                username,
                password,
            };
            list::main(&db_configuration, no_color).await
        }
        #[allow(deprecated)]
        Action::Branch(branch_args) => {
            let cli::BranchArgs { command } = branch_args;

            match command {
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
                        url: None,
                        ns,
                        db,
                        username,
                        password,
                    };
                    branch::new::main(name, &db_configuration).await
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
                        url: None,
                        ns,
                        db,
                        username,
                        password,
                    };
                    branch::remove::main(name, &db_configuration).await
                }
                None => Err(anyhow!("No action specified for `branch` command")),
            }
        }
    }
}
