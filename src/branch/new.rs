use color_eyre::eyre::{eyre, ContextCompat, Result};
use include_dir::{include_dir, Dir};
use names::{Generator, Name};
use std::path::{Path, PathBuf};
use surrealdb::{engine::any::Any, sql::Datetime, Surreal};

use crate::{
    branch::{
        common::{
            create_branching_feature_client, remove_dump_file, retrieve_existing_branch_names,
        },
        constants::BRANCH_NS,
    },
    config,
    input::SurrealdbConfiguration,
    io,
    models::Branch,
    surrealdb::create_surrealdb_client,
};

use super::{
    common::{create_branch_client, create_origin_branch_client},
    constants::{BRANCH_TABLE, DUMP_FILENAME},
};

pub struct NewBranchArgs<'a> {
    pub name: Option<String>,
    pub db_configuration: &'a SurrealdbConfiguration,
    pub config_file: Option<&'a Path>,
}

pub async fn main(args: NewBranchArgs<'_>) -> Result<()> {
    let NewBranchArgs {
        name,
        db_configuration,
        config_file,
    } = args;

    let db_config = config::retrieve_db_config(config_file);
    let db_configuration = merge_db_config(db_configuration, &db_config);

    let folder_path = config::retrieve_folder_path(config_file);
    let dump_file_path = io::concat_path(&folder_path, DUMP_FILENAME);

    let branching_feature_client =
        create_branching_feature_client(config_file, &db_configuration).await?;
    execute_schema_changes(&branching_feature_client).await?;

    let existing_branch_names = retrieve_existing_branch_names(&branching_feature_client).await?;

    fails_if_branch_already_exists(name.to_owned(), &existing_branch_names)?;

    export_original_branch_data_in_dump_file(
        config_file,
        &db_configuration,
        dump_file_path.to_owned(),
    )
    .await?;

    let result = replicate_database_into_branch(
        config_file,
        name,
        existing_branch_names,
        &db_configuration,
        branching_feature_client,
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

fn merge_db_config(
    db_configuration: &SurrealdbConfiguration,
    db_config: &config::DbConfig,
) -> SurrealdbConfiguration {
    SurrealdbConfiguration {
        address: db_configuration
            .address
            .to_owned()
            .or(db_config.address.to_owned()),
        username: db_configuration
            .username
            .to_owned()
            .or(db_config.username.to_owned()),
        password: db_configuration
            .password
            .to_owned()
            .or(db_config.password.to_owned()),
        ns: db_configuration.ns.to_owned().or(db_config.ns.to_owned()),
        db: db_configuration.db.to_owned().or(db_config.db.to_owned()),
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
    existing_branch_names: &[String],
) -> Result<()> {
    if let Some(name) = branch_name {
        if existing_branch_names.contains(&name) {
            return Err(eyre!("Branch name already exists"));
        }
    }

    Ok(())
}

async fn export_original_branch_data_in_dump_file(
    config_file: Option<&Path>,
    db_configuration: &SurrealdbConfiguration,
    dump_file_path: PathBuf,
) -> Result<()> {
    let client = create_surrealdb_client(config_file, db_configuration).await?;
    client.export(dump_file_path).await?;

    Ok(())
}

async fn replicate_database_into_branch(
    config_file: Option<&Path>,
    name: Option<String>,
    existing_branch_names: Vec<String>,
    db_configuration: &SurrealdbConfiguration,
    branching_feature_client: Surreal<Any>,
    dump_file_path: PathBuf,
) -> Result<String> {
    let branch_name = match name {
        Some(name) => name,
        None => generate_random_branch_name(existing_branch_names)?,
    };

    import_branch_data_from_dump_file(config_file, &branch_name, db_configuration, dump_file_path)
        .await?;

    save_branch_in_database(
        branch_name.to_string(),
        branching_feature_client,
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
    config_file: Option<&Path>,
    branch_name: &String,
    db_configuration: &SurrealdbConfiguration,
    dump_file_path: PathBuf,
) -> Result<()> {
    let client = create_branch_client(config_file, branch_name, db_configuration).await?;
    client.import(&dump_file_path).await?;

    let client = create_origin_branch_client(config_file, branch_name, db_configuration).await?;
    client.import(&dump_file_path).await?;

    Ok(())
}

async fn save_branch_in_database(
    branch_name: String,
    branching_feature_client: Surreal<Any>,
    db_configuration: &SurrealdbConfiguration,
) -> Result<()> {
    let record_key: (&str, &String) = (BRANCH_TABLE, &branch_name);
    let record: Option<Branch> = branching_feature_client
        .create(record_key)
        .content(Branch {
            name: branch_name.to_string(),
            from_ns: db_configuration.ns.clone().unwrap_or("test".to_owned()),
            from_db: db_configuration.db.clone().unwrap_or("test".to_owned()),
            created_at: Datetime::default(),
        })
        .await?;

    if record.is_none() {
        return Err(eyre!("Cannot insert branch name into branch table"));
    }

    Ok(())
}
