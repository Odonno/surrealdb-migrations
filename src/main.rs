use clap::Parser;
use cli::{Action, Args, CreateAction};
use create::CreateOperation;

mod apply;
mod cli;
mod create;
mod scaffold;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    match args.command {
        Action::Scaffold { kind } => scaffold::main(kind),
        Action::Create { command, name } => match name {
            Some(name) => create::main(name, CreateOperation::Migration),
            None => match command {
                Some(CreateAction::Schema { name }) => create::main(name, CreateOperation::Schema),
                Some(CreateAction::Event { name }) => create::main(name, CreateOperation::Event),
                Some(CreateAction::Migration { name }) => {
                    create::main(name, CreateOperation::Migration)
                }
                None => panic!("No action specified for `create` command"),
            },
        },
        Action::Update => todo!("Update migration(s) definitions"),
        Action::Apply {
            url,
            ns,
            db,
            username,
            password,
        } => apply::main(url, ns, db, username, password).await,
    };
}
