mod args;
mod diff_symbol;
mod table_diff;

pub use args::*;
use color_eyre::eyre::Result;
use futures::future::join_all;
use itertools::Itertools;
use lexicmp::natural_lexical_cmp;
use std::{collections::HashSet, path::Path};
use table_diff::TableDiff;

use crate::{
    apply::ensures_necessary_files_exists,
    constants::ALL_TAGS,
    input::SurrealdbConfiguration,
    io,
    models::MigrationDirection,
    runbin::surrealdb::create_surrealdb_client,
    surrealdb::{
        get_surrealdb_table_definition, get_surrealdb_table_definitions, parse_statements,
    },
};

pub async fn main(args: DiffArgs<'_>) -> Result<()> {
    let DiffArgs {
        no_color,
        config_file,
    } = args;

    if !no_color {
        owo_colors::set_override(false);
    }

    let local_statements = get_local_statements(config_file)?;
    let remote_statements = get_remote_statements(config_file).await?;

    let local_table_names = local_statements.iter().filter_map(map_table_name);
    let remote_table_names = remote_statements.iter().filter_map(map_table_name);

    let table_names: HashSet<String> =
        HashSet::from_iter(local_table_names.chain(remote_table_names));

    let mut table_diffs = Vec::with_capacity(table_names.len());

    for table_name in table_names {
        let remote_field_definitions = remote_statements
            .iter()
            .filter_map(map_define_field_statement);
        let remote_field_definitions = remote_field_definitions.filter(|s| s.what.0 == table_name);

        let local_field_definitions = local_statements
            .iter()
            .filter_map(map_define_field_statement);
        let local_field_definitions = local_field_definitions.filter(|s| s.what.0 == table_name);

        let local_fields: HashSet<String> =
            HashSet::from_iter(local_field_definitions.clone().map(|s| s.name.to_string()));
        let remote_fields: HashSet<String> =
            HashSet::from_iter(remote_field_definitions.clone().map(|s| s.name.to_string()));

        let only_local_fields = local_fields.difference(&remote_fields);
        let only_remote_fields = remote_fields
            .difference(&local_fields)
            // 💡 exclude certains fields automatically added by the server
            .filter(|f| !f.contains("[*]"))
            .filter(|f| f != &"in")
            .filter(|f| f != &"out");

        let additions = HashSet::from_iter(only_local_fields.cloned());
        let deletions = HashSet::from_iter(only_remote_fields.cloned());

        let changes = remote_field_definitions.filter(|remote_d| {
            let field_name = remote_d.name.to_string();
            let local_d = local_field_definitions
                .clone()
                .find(|ld| ld.name.to_string() == field_name);

            let Some(local_d) = local_d else {
                return false;
            };

            remote_d.readonly != local_d.readonly
                || remote_d.assert != local_d.assert
                || remote_d.default != local_d.default
                || remote_d.kind != local_d.kind
                || remote_d.permissions != local_d.permissions
                || remote_d.reference != local_d.reference
        });
        let changes = HashSet::from_iter(changes.map(|s| s.name.to_string()));

        if !additions.is_empty() || !changes.is_empty() || !deletions.is_empty() {
            let table_diff = TableDiff {
                name: table_name.to_string(),
                additions,
                changes,
                deletions,
            };
            table_diffs.push(table_diff);
        }
    }

    if table_diffs.is_empty() {
        println!("No changes detected.");
    } else {
        for table_diff in table_diffs
            .iter()
            .sorted_by(|a, b| natural_lexical_cmp(&a.name, &b.name))
        {
            println!("{}", table_diff);
        }
    }

    Ok(())
}

fn get_local_statements(config_file: Option<&Path>) -> Result<Vec<::surrealdb::sql::Statement>> {
    let tags = HashSet::from([ALL_TAGS.into()]);
    let exclude_tags = HashSet::new();

    let schemas_files = io::extract_schemas_files(config_file, None, &tags, &exclude_tags)
        .ok()
        .unwrap_or_default();
    let events_files = io::extract_events_files(config_file, None, &tags, &exclude_tags)
        .ok()
        .unwrap_or_default();

    let schema_definitions = io::concat_files_content(&schemas_files);
    let event_definitions = io::concat_files_content(&events_files);

    let forward_migrations_files = io::extract_migrations_files(
        config_file,
        None,
        MigrationDirection::Forward,
        &tags,
        &exclude_tags,
    );

    let use_traditional_approach = schema_definitions.is_empty()
        && event_definitions.is_empty()
        && !forward_migrations_files.is_empty();

    ensures_necessary_files_exists(
        use_traditional_approach,
        &schemas_files,
        &forward_migrations_files,
    )?;

    let use_traditional_approach = schema_definitions.is_empty()
        && event_definitions.is_empty()
        && !forward_migrations_files.is_empty();

    let local_statements = if use_traditional_approach {
        todo!("Extracting schema definition in traditional approach is not yet available")
    } else {
        let schemas_statements = parse_statements(&schema_definitions)?;
        let events_statements = parse_statements(&event_definitions)?;

        schemas_statements
            .into_iter()
            .chain(events_statements)
            .collect_vec()
    };

    Ok(local_statements)
}

async fn get_remote_statements(
    config_file: Option<&Path>,
) -> Result<Vec<::surrealdb::sql::Statement>> {
    let client = create_surrealdb_client(config_file, &SurrealdbConfiguration::default()).await?;

    let table_definitions = get_surrealdb_table_definitions(&client).await?;

    let futures = table_definitions
        .keys()
        .map(|table_name| get_surrealdb_table_definition(&client, table_name));

    let inner_table_definitions: Result<Vec<_>, _> = join_all(futures).await.into_iter().collect();
    let inner_table_definitions = inner_table_definitions?;

    let remote_statements = inner_table_definitions
        .into_iter()
        .flat_map(|d| d.fields.values().cloned().collect_vec())
        .chain(table_definitions.clone().into_values());
    let remote_statements: Result<Vec<_>, _> =
        remote_statements.map(|s| parse_statements(&s)).collect();
    let remote_statements = remote_statements?;
    let remote_statements = remote_statements
        .into_iter()
        .flat_map(|q| q.0)
        .collect_vec();

    Ok(remote_statements)
}

fn map_table_name(statement: &::surrealdb::sql::Statement) -> Option<String> {
    match statement {
        ::surrealdb::sql::Statement::Define(define_statement) => match define_statement {
            ::surrealdb::sql::statements::DefineStatement::Table(statement) => {
                Some(statement.name.0.to_string())
            }
            ::surrealdb::sql::statements::DefineStatement::Field(statement) => {
                Some(statement.what.0.to_string())
            }
            _ => None,
        },
        _ => None,
    }
}

fn map_define_field_statement(
    statement: &::surrealdb::sql::Statement,
) -> Option<::surrealdb::sql::statements::DefineFieldStatement> {
    match statement {
        ::surrealdb::sql::Statement::Define(
            ::surrealdb::sql::statements::DefineStatement::Field(define_field_statement),
        ) => Some(define_field_statement.clone()),
        _ => None,
    }
}
