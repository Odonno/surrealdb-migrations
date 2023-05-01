use anyhow::Result;

use crate::{cli::ScaffoldTemplate, config};

use super::common::{
    apply_after_scaffold, apply_before_scaffold, copy_template_files_to_current_dir,
};

pub fn main(template: ScaffoldTemplate) -> Result<()> {
    let folder_path = config::retrieve_folder_path();

    apply_before_scaffold(folder_path.to_owned())?;

    copy_template_files_to_current_dir(template, folder_path.to_owned())?;

    apply_after_scaffold(folder_path.to_owned())?;

    Ok(())
}
