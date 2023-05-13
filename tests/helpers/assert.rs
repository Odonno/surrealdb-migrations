use anyhow::{anyhow, Result};
use std::path::Path;

pub fn are_folders_equivalent(folder_one: &str, folder_two: &str) -> Result<bool> {
    let is_different = dir_diff::is_different(folder_one, folder_two);

    match is_different {
        Ok(is_different) => {
            let are_equivalent = !is_different;
            Ok(are_equivalent)
        }
        Err(error) => Err(anyhow!("Cannot compare folders. {:?}", error)),
    }
}

pub fn is_empty_folder(folder: &str) -> Result<bool> {
    let dir = Path::new(folder).read_dir()?;
    let nubmer_of_files = dir.count();

    Ok(nubmer_of_files == 0)
}

pub fn is_file_exists(file_path: &str) -> Result<bool> {
    let file = Path::new(file_path);
    let is_file_exists = file.try_exists()?;

    Ok(is_file_exists)
}
