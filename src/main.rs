use anyhow::{anyhow, Result};
use apply::ApplyArgs;
use clap::Parser;
use cli::{Action, Args, CreateAction, ScaffoldAction};
use create::{CreateArgs, CreateEventArgs, CreateMigrationArgs, CreateOperation, CreateSchemaArgs};
use input::SurrealdbConfiguration;

mod apply;
mod cli;
mod config;
mod constants;
mod create;
mod definitions;
mod input;
mod list;
mod models;
mod remove;
mod scaffold;
mod surrealdb;

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
        Action::Create {
            command,
            name,
            down,
        } => match name {
            Some(name) => {
                let operation = CreateOperation::Migration(CreateMigrationArgs { down });
                let args = CreateArgs { name, operation };
                create::main(args)
            }
            None => match command {
                Some(CreateAction::Schema {
                    name,
                    fields,
                    dry_run,
                }) => {
                    let operation = CreateOperation::Schema(CreateSchemaArgs { fields, dry_run });
                    let args = CreateArgs { name, operation };
                    create::main(args)
                }
                Some(CreateAction::Event {
                    name,
                    fields,
                    dry_run,
                }) => {
                    let operation = CreateOperation::Event(CreateEventArgs { fields, dry_run });
                    let args = CreateArgs { name, operation };
                    create::main(args)
                }
                Some(CreateAction::Migration { name, down }) => {
                    let operation = CreateOperation::Migration(CreateMigrationArgs { down });
                    let args = CreateArgs { name, operation };
                    create::main(args)
                }
                None => Err(anyhow!("No action specified for `create` command")),
            },
        },
        Action::Remove {} => remove::main(),
        Action::Apply {
            up,
            down,
            url,
            ns,
            db,
            username,
            password,
            dry_run,
        } => {
            let db_configuration = SurrealdbConfiguration {
                url,
                ns,
                db,
                username,
                password,
            };
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
            let args = ApplyArgs {
                operation,
                db_configuration: &db_configuration,
                display_logs: true,
                dry_run,
            };
            apply::main(args).await
        }
        Action::List {
            url,
            ns,
            db,
            username,
            password,
            no_color,
        } => {
            let db_configuration = SurrealdbConfiguration {
                url,
                ns,
                db,
                username,
                password,
            };
            list::main(&db_configuration, no_color).await
        }
    }
}
