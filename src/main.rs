use clap::Parser;
use cli::{Action, Args};

mod cli;
mod scaffold;

fn main() {
    let args = Args::parse();

    match args.command {
        Action::Scaffold { kind } => scaffold::main(kind),
        Action::Create { name } => todo!("Create a new migration file: {}", name),
        Action::Update => todo!("Update migration(s) definitions"),
        Action::Apply {
            url,
            ns,
            db,
            username,
            password,
        } => todo!(
            "Apply migration(s) to the database: {:?} {:?} {:?} {:?} {:?}",
            url,
            ns,
            db,
            username,
            password
        ),
    };
}
