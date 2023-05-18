use anyhow::{Context, Result};
use ini::Ini;
use std::path::Path;

pub fn set_config_value(section: &str, key: &str, value: &str) -> Result<()> {
    let mut config = load_config()?;
    let section = config
        .section_mut(Some(section))
        .context("Could not find section")?;

    section.insert(key, value);

    config.write_to_file(".surrealdb")?;

    Ok(())
}

pub fn reset_config() -> Result<()> {
    let surrealdb_config_file = Path::new(".surrealdb");

    let default_value = r#"[core]
    path = "./tests-files"
    schema = "less"

[db]
    address = "ws://localhost:8000"
    username = "root"
    password = "root"
    ns = "test"
    db = "test""#;

    if surrealdb_config_file.exists() {
        std::fs::write(surrealdb_config_file, default_value)?;
    }

    Ok(())
}

fn load_config() -> Result<Ini> {
    let surrealdb_config_file = Path::new(".surrealdb");
    let ini = Ini::load_from_file(surrealdb_config_file)?;

    Ok(ini)
}
