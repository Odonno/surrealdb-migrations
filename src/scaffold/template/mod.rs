mod args;

pub use args::ScaffoldFromTemplateArgs;
use color_eyre::eyre::Result;

use crate::config;

use super::common::{
    apply_after_scaffold, apply_before_scaffold, copy_template_files_to_current_dir,
};

pub fn main(args: ScaffoldFromTemplateArgs) -> Result<()> {
    let ScaffoldFromTemplateArgs {
        template,
        traditional,
        config_file,
    } = args;

    let folder_path = config::retrieve_folder_path(config_file);

    apply_before_scaffold(folder_path.to_owned())?;

    copy_template_files_to_current_dir(template, folder_path.to_owned())?;

    apply_after_scaffold(config_file, traditional, folder_path)?;

    Ok(())
}
