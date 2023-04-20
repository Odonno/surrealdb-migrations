pub fn clear_files_dir() {
    let files_dir = std::path::Path::new("tests-files");

    if files_dir.exists() {
        std::fs::remove_dir_all(files_dir).unwrap();
    }
}
