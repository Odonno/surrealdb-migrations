use clap::Parser;
use cli::{Action, Args};

mod cli;

fn main() {
    let args = Args::parse();

    match args.command {
        Action::Scaffold { kind } => todo!("Scaffold a new SurrealDB project: {:?}", kind),
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
