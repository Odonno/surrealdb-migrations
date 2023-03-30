use clap::Parser;
use cli::{Action, Args, CreateAction};
use create::CreateOperation;

mod apply;
mod cli;
mod config;
mod create;
mod definitions;
mod scaffold;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    match args.command {
        Action::Scaffold { template } => scaffold::main(template),
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
                None => panic!("No action specified for `create` command"),
            },
        },
        Action::Apply {
            up,
            url,
            ns,
            db,
            username,
            password,
        } => apply::main(up, url, ns, db, username, password).await,
    };
}
