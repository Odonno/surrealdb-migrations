use color_eyre::eyre::{eyre, Result};
use std::path::Path;

use crate::{cli, input::SurrealdbConfiguration};

use super::{
    diff::BranchDiffArgs, list::ListBranchArgs, merge::MergeBranchArgs, new::NewBranchArgs,
    remove::RemoveBranchArgs, status::BranchStatusArgs,
};

pub enum BranchArgs<'a> {
    Diff(BranchDiffArgs<'a>),
    List(ListBranchArgs<'a>),
    Merge(MergeBranchArgs<'a>),
    New(NewBranchArgs<'a>),
    Remove(RemoveBranchArgs<'a>),
    Status(BranchStatusArgs<'a>),
}

impl<'a> BranchArgs<'a> {
    pub fn try_from(value: cli::BranchArgs, config_file: Option<&'a Path>) -> Result<Self> {
        let cli::BranchArgs { command, name } = value;

        match name {
            Some(name) => Ok(BranchArgs::Status(BranchStatusArgs { name, config_file })),
            None => match command {
                Some(cli::BranchAction::New {
                    name,
                    address,
                    ns,
                    db,
                    username,
                    password,
                }) => {
                    let db_configuration = SurrealdbConfiguration {
                        address,
                        ns,
                        db,
                        username,
                        password,
                    };
                    Ok(BranchArgs::New(NewBranchArgs {
                        name,
                        db_configuration,
                        config_file,
                    }))
                }
                Some(cli::BranchAction::Remove {
                    name,
                    address,
                    ns,
                    db,
                    username,
                    password,
                }) => {
                    let db_configuration = SurrealdbConfiguration {
                        address,
                        ns,
                        db,
                        username,
                        password,
                    };
                    Ok(BranchArgs::Remove(RemoveBranchArgs {
                        name,
                        db_configuration,
                        config_file,
                    }))
                }
                Some(cli::BranchAction::Merge {
                    name,
                    mode,
                    address,
                    ns,
                    db,
                    username,
                    password,
                }) => {
                    let db_configuration = SurrealdbConfiguration {
                        address,
                        ns,
                        db,
                        username,
                        password,
                    };
                    Ok(BranchArgs::Merge(MergeBranchArgs {
                        name,
                        mode,
                        db_configuration,
                        config_file,
                    }))
                }
                Some(cli::BranchAction::Status { name }) => {
                    Ok(BranchArgs::Status(BranchStatusArgs { name, config_file }))
                }
                Some(cli::BranchAction::List {
                    address,
                    ns,
                    db,
                    username,
                    password,
                    no_color,
                }) => {
                    let db_configuration = SurrealdbConfiguration {
                        address,
                        ns,
                        db,
                        username,
                        password,
                    };
                    Ok(BranchArgs::List(ListBranchArgs {
                        db_configuration,
                        no_color,
                        config_file,
                    }))
                }
                Some(cli::BranchAction::Diff {
                    name,
                    address,
                    ns,
                    db,
                    username,
                    password,
                }) => {
                    let db_configuration = SurrealdbConfiguration {
                        address,
                        ns,
                        db,
                        username,
                        password,
                    };
                    Ok(BranchArgs::Diff(BranchDiffArgs {
                        name,
                        db_configuration,
                        config_file,
                    }))
                }
                None => Err(eyre!("No action specified for `branch` command")),
            },
        }
    }
}
