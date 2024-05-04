use color_eyre::eyre::{eyre, ContextCompat, Result};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};

use crate::{input::SurrealdbConfiguration, models::Branch, surrealdb::create_surrealdb_client};

use super::{
    common::{create_branching_feature_client, get_branch_table},
    constants::{BRANCH_NS, ORIGIN_BRANCH_NS},
};

pub struct BranchDiffArgs<'a> {
    pub name: String,
    pub db_configuration: &'a SurrealdbConfiguration,
    pub config_file: Option<&'a Path>,
}

pub async fn main(args: BranchDiffArgs<'_>) -> Result<()> {
    let BranchDiffArgs {
        name,
        db_configuration,
        config_file,
    } = args;

    let branching_feature_client =
        create_branching_feature_client(config_file, db_configuration).await?;
    let branch: Option<Branch> = get_branch_table(&branching_feature_client, &name).await?;

    match branch {
        Some(branch) => {
            // Retrieve branch table definitions from the 2 branches
            #[allow(deprecated)]
            let branch_db_configuration = SurrealdbConfiguration {
                address: db_configuration.address.clone(),
                url: db_configuration.url.clone(),
                username: db_configuration.username.clone(),
                password: db_configuration.password.clone(),
                ns: Some(BRANCH_NS.to_owned()),
                db: Some(branch.name.to_owned()),
            };
            let branch_table_definitions =
                get_surrealdb_database_definition(config_file, branch_db_configuration).await?;

            #[allow(deprecated)]
            let origin_branch_db_configuration = SurrealdbConfiguration {
                address: db_configuration.address.clone(),
                url: db_configuration.url.clone(),
                username: db_configuration.username.clone(),
                password: db_configuration.password.clone(),
                ns: Some(ORIGIN_BRANCH_NS.to_owned()),
                db: Some(branch.name.to_owned()),
            };
            let origin_branch_table_definitions =
                get_surrealdb_database_definition(config_file, origin_branch_db_configuration)
                    .await?;

            // Compare branch table definitions
            let origin_tables = origin_branch_table_definitions.keys().collect::<Vec<_>>();
            let branch_tables = branch_table_definitions.keys().collect::<Vec<_>>();

            let tables_created = branch_tables
                .clone()
                .into_iter()
                .filter(|branch_table| !origin_tables.contains(branch_table))
                .collect::<Vec<_>>();
            let tables_altered = origin_tables
                .clone()
                .into_iter()
                .filter(|origin_table| {
                    let origin_table_definition = origin_branch_table_definitions
                        .get(&origin_table.to_string())
                        .context("Cannot retrieve 'altered' table definition")
                        .ok();
                    let branch_table_definition = branch_table_definitions
                        .get(&origin_table.to_string())
                        .context("Cannot retrieve 'altered' table definition")
                        .ok();

                    if origin_table_definition.is_none() || branch_table_definition.is_none() {
                        return false;
                    }

                    origin_table_definition.unwrap() != branch_table_definition.unwrap()
                })
                .collect::<Vec<_>>();
            let tables_removed = origin_tables
                .clone()
                .into_iter()
                .filter(|origin_table| !branch_tables.contains(origin_table))
                .collect::<Vec<_>>();

            let has_created_tables = !tables_created.is_empty();
            let has_altered_tables = !tables_altered.is_empty();
            let has_removed_tables = !tables_removed.is_empty();

            let has_schemas_changes =
                has_created_tables || has_altered_tables || has_removed_tables;

            if has_schemas_changes {
                println!("Schema changes detected:");

                if has_created_tables {
                    println!();
                    println!("### {} tables created ###", tables_created.len());

                    for table_created in tables_created {
                        let definition = branch_table_definitions
                            .get(table_created)
                            .context("Cannot retrieve 'created' table definition")?;

                        println!();
                        println!("## {} ##", table_created);
                        println!();
                        println!("{}", definition);
                    }
                }
                if has_altered_tables {
                    println!();
                    println!("### {} tables altered ###", tables_altered.len());

                    for table_altered in tables_altered {
                        let origin_definition = origin_branch_table_definitions
                            .get(table_altered)
                            .context("Cannot retrieve 'altered' table definition")?;
                        let branch_definition = branch_table_definitions
                            .get(table_altered)
                            .context("Cannot retrieve 'altered' table definition")?;

                        let diff_definition =
                            diffy::create_patch(origin_definition, branch_definition).to_string();

                        println!();
                        println!("## {} ##", table_altered);
                        println!();
                        println!("{}", diff_definition);
                    }
                }
                if has_removed_tables {
                    println!();
                    println!("### {} tables removed ###", tables_removed.len());

                    for table_removed in tables_removed {
                        let definition = origin_branch_table_definitions
                            .get(table_removed)
                            .context("Cannot retrieve 'removed' table definition")?;

                        println!();
                        println!("## {} ##", table_removed);
                        println!();
                        println!("{}", definition);
                    }
                }
            } else {
                println!("No schema changes detected");
            }
        }
        None => {
            return Err(eyre!("Branch {} does not exist", name));
        }
    }

    Ok(())
}

type SurrealdbTableDefinitions = HashMap<String, String>;
type SurrealdbEventDefinitions = HashMap<String, String>;
type SurrealdbFieldDefinitions = HashMap<String, String>;
type SurrealdbForeignTableDefinitions = HashMap<String, String>;
type SurrealdbIndexDefinitions = HashMap<String, String>;
type SurrealdbDatabaseDefinition = HashMap<String, String>;

#[derive(Serialize, Deserialize, Debug)]
struct SurrealdbInfoForTableResponse {
    events: SurrealdbEventDefinitions,
    fields: SurrealdbFieldDefinitions,
    tables: SurrealdbForeignTableDefinitions,
    indexes: SurrealdbIndexDefinitions,
}

async fn get_surrealdb_database_definition(
    config_file: Option<&Path>,
    db_configuration: SurrealdbConfiguration,
) -> Result<SurrealdbDatabaseDefinition> {
    let client = create_surrealdb_client(config_file, &db_configuration).await?;

    const DATABASE_DEFINITION_QUERY: &str = "INFO FOR DB;";
    let mut response = client.query(DATABASE_DEFINITION_QUERY).await?;

    let result: Option<SurrealdbTableDefinitions> = response.take("tables")?;
    let table_definitions = result.context("Failed to get table definitions")?;

    let tables = table_definitions.keys().collect::<Vec<_>>();

    let table_definitions_query = tables
        .iter()
        .map(|table| format!("INFO FOR TABLE {};", table))
        .collect::<Vec<_>>()
        .join("\n");
    let mut response = client.query(table_definitions_query).await?;

    let database_definition = tables
        .iter()
        .enumerate()
        .map(|(index, table)| -> Result<(String, String)> {
            let table_definition = table_definitions
                .get(&table.to_string())
                .context("Failed to get table definition")?;

            let info_for_table_response: Option<SurrealdbInfoForTableResponse> =
                response.take(index)?;
            let info_for_table_response = info_for_table_response
                .context(format!("Failed to get info for table {}", table))?;

            let mut full_definition = vec![table_definition.to_string()];
            for value in info_for_table_response.events.values().sorted() {
                full_definition.push(value.to_string());
            }
            for value in info_for_table_response.fields.values().sorted() {
                full_definition.push(value.to_string());
            }
            for value in info_for_table_response.tables.values().sorted() {
                full_definition.push(value.to_string());
            }
            for value in info_for_table_response.indexes.values().sorted() {
                full_definition.push(value.to_string());
            }
            let full_definition = full_definition.join("\n");

            Ok((table.to_string(), full_definition))
        })
        .collect::<Result<SurrealdbDatabaseDefinition>>()?;

    Ok(database_definition)
}
