use color_eyre::eyre::{Result};

use crate::{cli::ScaffoldTemplate, config};

use super::common::{
    apply_after_scaffold, apply_before_scaffold, copy_template_files_to_current_dir,
};

pub struct ScaffoldFromTemplateArgs<'a> {
    pub template: ScaffoldTemplate,
    pub config_file: Option<&'a str>,
}

pub fn main(args: ScaffoldFromTemplateArgs) -> Result<()> {
    let ScaffoldFromTemplateArgs {
        template,
        config_file,
    } = args;

    let folder_path = config::retrieve_folder_path(config_file);

    apply_before_scaffold(folder_path.to_owned())?;

    copy_template_files_to_current_dir(template, folder_path.to_owned())?;

    apply_after_scaffold(folder_path)?;

    Ok(())
}
