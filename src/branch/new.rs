use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use include_dir::{include_dir, Dir};
use names::{Generator, Name};
use surrealdb::{engine::any::Any, Surreal};

use crate::{
    branch::{
        common::{create_branch_data_client, remove_dump_file, retrieve_existing_branch_names},
        constants::BRANCH_NS,
    },
    config,
    input::SurrealdbConfiguration,
    io,
    models::Branch,
    surrealdb::create_surrealdb_client,
};

use super::{
    common::create_branch_client,
    constants::{BRANCH_TABLE, DUMP_FILENAME},
};

pub async fn main(name: Option<String>, db_configuration: &SurrealdbConfiguration) -> Result<()> {
    let folder_path = config::retrieve_folder_path();
    let dump_file_path = io::concat_path(&folder_path, DUMP_FILENAME);

    let branch_data_client = create_branch_data_client(db_configuration).await?;
    execute_schema_changes(&branch_data_client).await?;

    let existing_branch_names = retrieve_existing_branch_names(&branch_data_client).await?;

    fails_if_branch_already_exists(name.to_owned(), &existing_branch_names)?;

    export_original_branch_data_in_dump_file(db_configuration, dump_file_path.to_owned()).await?;

    let result = replicate_database_into_branch(
        name,
        existing_branch_names,
        db_configuration,
        branch_data_client,
        dump_file_path.to_owned(),
    )
    .await;

    match result {
        Ok(branch_name) => {
            remove_dump_file(&dump_file_path)?;

            println!("You can now use the branch with the following configuration:\n");
            println!("ns: {}", BRANCH_NS);
            println!("db: {}", branch_name);

            Ok(())
        }
        Err(error) => {
            remove_dump_file(&dump_file_path)?;

            Err(error)
        }
    }
}

async fn execute_schema_changes(branch_data_client: &Surreal<Any>) -> Result<()> {
    const SCHEMAS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/branch/schemas");

    let branch_schema = SCHEMAS_DIR
        .get_file("branch.surql")
        .context("Cannot extract branch.surql file")?
        .contents_utf8()
        .context("Cannot get content of branch.surql file")?;

    branch_data_client.query(branch_schema).await?;

    Ok(())
}

fn fails_if_branch_already_exists(
    branch_name: Option<String>,
    existing_branch_names: &Vec<String>,
) -> Result<()> {
    if let Some(name) = branch_name {
        if existing_branch_names.contains(&name) {
            return Err(anyhow!("Branch name already exists"));
        }
    }

    Ok(())
}

async fn export_original_branch_data_in_dump_file(
    db_configuration: &SurrealdbConfiguration,
    dump_file_path: PathBuf,
) -> Result<()> {
    let client = create_surrealdb_client(&db_configuration).await?;
    client.export(dump_file_path).await?;

    Ok(())
}

async fn replicate_database_into_branch(
    name: Option<String>,
    existing_branch_names: Vec<String>,
    db_configuration: &SurrealdbConfiguration,
    branch_data_client: Surreal<Any>,
    dump_file_path: PathBuf,
) -> Result<String> {
    let branch_name = match name {
        Some(name) => name,
        None => generate_random_branch_name(existing_branch_names)?,
    };

    import_branch_data_from_dump_file(&branch_name, db_configuration, dump_file_path).await?;

    save_branch_in_database(
        branch_name.to_string(),
        branch_data_client,
        db_configuration,
    )
    .await?;

    Ok(branch_name)
}

fn generate_random_branch_name(existing_branch_names: Vec<String>) -> Result<String> {
    let mut branch_name: String;
    let mut generator = Generator::with_naming(Name::Numbered);

    loop {
        branch_name = generator.next().context("Cannot generate branch name")?;

        if !existing_branch_names.contains(&branch_name) {
            break;
        }
    }

    Ok(branch_name)
}

async fn import_branch_data_from_dump_file(
    branch_name: &String,
    db_configuration: &SurrealdbConfiguration,
    dump_file_path: PathBuf,
) -> Result<()> {
    let client = create_branch_client(branch_name, &db_configuration).await?;
    client.import(dump_file_path).await?;

    Ok(())
}

async fn save_branch_in_database(
    branch_name: String,
    branch_data_client: Surreal<Any>,
    db_configuration: &SurrealdbConfiguration,
) -> Result<()> {
    let record_key: (&str, &String) = (BRANCH_TABLE, &branch_name);
    let record: Option<Branch> = branch_data_client
        .create(record_key)
        .content(Branch {
            name: branch_name.to_string(),
            from_ns: db_configuration.ns.clone().unwrap_or("test".to_owned()),
            from_db: db_configuration.db.clone().unwrap_or("test".to_owned()),
            created_at: chrono::Utc::now().to_rfc3339(),
        })
        .await?;

    if record.is_none() {
        return Err(anyhow!("Cannot insert branch name into branch table"));
    }

    Ok(())
}
