use anyhow::{anyhow, Result};
use clap::Parser;
use cli::{Action, Args, CreateAction, ScaffoldAction};
use create::CreateOperation;

mod apply;
mod cli;
mod config;
mod constants;
mod create;
mod definitions;
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
            ScaffoldAction::Template { template } => scaffold::from_template(template),
            ScaffoldAction::Schema {
                schema,
                db_type,
                preserve_casing,
            } => scaffold::from_schema(schema, db_type, preserve_casing),
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
        } => apply::execute(up, url, ns, db, username, password, true).await,
        Action::List {
            url,
            ns,
            db,
            username,
            password,
            no_color,
        } => list::main(url, ns, db, username, password, no_color).await,
    }
}
