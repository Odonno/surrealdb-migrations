use color_eyre::{eyre::eyre, Result};
use std::path::Path;

pub fn are_folders_equivalent(path_one: &Path, path_two: &Path) -> Result<bool> {
    let is_different = dir_diff::is_different(path_one, path_two);

    match is_different {
        Ok(is_different) => {
            let are_equivalent = !is_different;
            Ok(are_equivalent)
        }
        Err(error) => Err(eyre!("Cannot compare folders. {:?}", error)),
    }
}

pub fn is_empty_folder(path: &Path) -> Result<bool> {
    let dir = path.read_dir()?;
    let nubmer_of_files = dir.count();

    Ok(nubmer_of_files == 0)
}

pub fn is_folder_exists(path: &Path) -> Result<bool> {
    let exists = path.try_exists()?;
    Ok(exists)
}
