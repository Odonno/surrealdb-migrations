use std::path::Path;

use crate::config;

pub enum CreateOperation {
    Schema,
    Event,
    Migration,
}

pub fn main(name: String, operation: CreateOperation, fields: Option<Vec<String>>, dry_run: bool) {
    let folder_path = config::retrieve_folder_path();

    let dir_name = match operation {
        CreateOperation::Schema => "schemas",
        CreateOperation::Event => "events",
        CreateOperation::Migration => "migrations",
    };

    // retrieve folder path
    let folder_path = match folder_path.to_owned() {
        Some(folder_path) => {
            let path = Path::new(&folder_path);
            path.join(dir_name)
        }
        None => Path::new(dir_name).to_path_buf(),
    };

    let filename = match operation {
        CreateOperation::Schema => format!("{}.surql", name),
        CreateOperation::Event => format!("{}.surql", name),
        CreateOperation::Migration => {
            let now = chrono::Local::now();
            format!(
                "{}_{}_{}.surql",
                now.format("%Y%m%d"),
                now.format("%H%M%S"),
                name
            )
        }
    };

    let file_path = folder_path.join(&filename);

    if !dry_run {
        // check that directory exists
        if !folder_path.exists() {
            panic!("Directory {} doesn't exist", dir_name);
        }

        // check that file doesn't already exist

        if file_path.exists() {
            panic!("File {} already exists", filename);
        }
    }

    // generate content
    let field_definitions = match fields {
        Some(fields) => fields
            .iter()
            .map(|field| format!("DEFINE FIELD {} ON {};", field, name))
            .collect::<Vec<String>>()
            .join("\n"),
        None => format!("# DEFINE FIELD field ON {};", name),
    };

    let content = match operation {
        CreateOperation::Schema => format!(
            "DEFINE TABLE {0} SCHEMALESS;

{1}",
            name, field_definitions
        ),
        CreateOperation::Event => format!(
            "DEFINE TABLE {0} SCHEMALESS;

{1}

DEFINE EVENT {0} ON TABLE {0} WHEN $before == NONE THEN (
    # TODO
);",
            name, field_definitions
        ),
        CreateOperation::Migration => "".to_string(),
    };

    match dry_run {
        true => {
            println!("{}", content);
        }
        false => {
            // create file
            fs_extra::file::write_all(&file_path, &content).unwrap();

            println!("File {} created successfully", filename);
        }
    }
}
