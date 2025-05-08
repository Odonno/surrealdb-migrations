use color_eyre::eyre::{eyre, Result};
use std::path::Path;

use crate::cli;

pub struct CreateArgs<'a> {
    pub name: String,
    pub operation: CreateOperation,
    pub config_file: Option<&'a Path>,
}

pub enum CreateOperation {
    Schema(CreateSchemaArgs),
    Event(CreateEventArgs),
    Migration(CreateMigrationArgs),
}

pub struct CreateSchemaArgs {
    pub fields: Option<Vec<String>>,
    pub dry_run: bool,
    pub schemafull: bool,
}

pub struct CreateEventArgs {
    pub fields: Option<Vec<String>>,
    pub dry_run: bool,
    pub schemafull: bool,
}

pub struct CreateMigrationArgs {
    pub down: bool,
    pub content: Option<String>,
}

impl<'a> CreateArgs<'a> {
    pub fn try_from(value: cli::CreateArgs, config_file: Option<&'a Path>) -> Result<Self> {
        let cli::CreateArgs {
            command,
            name,
            down,
            content,
        } = value;

        match name {
            Some(name) => {
                let operation = CreateOperation::Migration(CreateMigrationArgs { down, content });
                Ok(CreateArgs {
                    name,
                    operation,
                    config_file,
                })
            }
            None => match command {
                Some(cli::CreateAction::Schema {
                    name,
                    fields,
                    dry_run,
                    schemafull,
                }) => {
                    let operation = CreateOperation::Schema(CreateSchemaArgs {
                        fields,
                        dry_run,
                        schemafull,
                    });
                    Ok(CreateArgs {
                        name,
                        operation,
                        config_file,
                    })
                }
                Some(cli::CreateAction::Event {
                    name,
                    fields,
                    dry_run,
                    schemafull,
                }) => {
                    let operation = CreateOperation::Event(CreateEventArgs {
                        fields,
                        dry_run,
                        schemafull,
                    });
                    Ok(CreateArgs {
                        name,
                        operation,
                        config_file,
                    })
                }
                Some(cli::CreateAction::Migration {
                    name,
                    down,
                    content,
                }) => {
                    let operation =
                        CreateOperation::Migration(CreateMigrationArgs { down, content });
                    Ok(CreateArgs {
                        name,
                        operation,
                        config_file,
                    })
                }
                None => Err(eyre!("No action specified for `create` command")),
            },
        }
    }
}
