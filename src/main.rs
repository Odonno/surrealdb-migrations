use anyhow::{anyhow, Result};
use apply::ApplyArgs;
use clap::Parser;
use cli::{Action, Args, CreateAction, ScaffoldAction};
use create::CreateOperation;
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
        Action::Create { command, name } => match name {
            Some(name) => create::main(name, CreateOperation::Migration, None, false),
            None => match command {
                Some(CreateAction::Schema {
                    name,
                    fields,
                    dry_run,
                }) => create::main(name, CreateOperation::Schema, fields, dry_run),
                Some(CreateAction::Event {
                    name,
                    fields,
                    dry_run,
                }) => create::main(name, CreateOperation::Event, fields, dry_run),
                Some(CreateAction::Migration { name }) => {
                    create::main(name, CreateOperation::Migration, None, false)
                }
                None => Err(anyhow!("No action specified for `create` command")),
            },
        },
        Action::Remove {} => remove::main(),
        Action::Apply {
            up,
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
            let args = ApplyArgs {
                up,
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
