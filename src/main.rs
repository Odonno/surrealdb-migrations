use clap::Parser;
use cli::{Action, Args, CreateAction};
use create::CreateOperation;

mod apply;
mod cli;
mod create;
mod definitions;
mod scaffold;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    match args.command {
        Action::Scaffold { template } => scaffold::main(template),
        Action::Create { command, name } => match name {
            Some(name) => create::main(name, CreateOperation::Migration, None),
            None => match command {
                Some(CreateAction::Schema { name, fields }) => {
                    create::main(name, CreateOperation::Schema, fields)
                }
                Some(CreateAction::Event { name, fields }) => {
                    create::main(name, CreateOperation::Event, fields)
                }
                Some(CreateAction::Migration { name }) => {
                    create::main(name, CreateOperation::Migration, None)
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
