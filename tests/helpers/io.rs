use anyhow::Result;

pub fn clear_tests_files() -> Result<()> {
    remove_folder("tests-files")?;

    Ok(())
}

pub fn empty_folder(folder: &str) -> Result<()> {
    let migrations_files_dir = std::path::Path::new(folder);

    if migrations_files_dir.exists() {
        std::fs::remove_dir_all(migrations_files_dir)?;
        std::fs::create_dir(migrations_files_dir)?;
    }

    Ok(())
}

pub fn remove_folder(folder: &str) -> Result<()> {
    let dir = std::path::Path::new(folder);

    if dir.exists() {
        std::fs::remove_dir_all(dir)?;
    }

    Ok(())
}

pub fn add_new_schema_file() -> Result<()> {
    let schemas_files_dir = std::path::Path::new("tests-files/schemas");

    if schemas_files_dir.exists() {
        let category_schema_file = schemas_files_dir.join("category.surql");
        const CATEGORY_CONTENT: &str = "DEFINE TABLE category SCHEMALESS;

DEFINE FIELD name ON category TYPE string;
DEFINE FIELD created_at ON comment TYPE datetime VALUE $before OR time::now();";

        std::fs::write(category_schema_file, CATEGORY_CONTENT)?;
    }

    Ok(())
}
