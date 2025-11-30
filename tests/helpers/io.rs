use color_eyre::eyre::{ContextCompat, Result};
use std::{fs, path::Path};

use super::create_cmd;

pub fn empty_folder(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(path)?;
        fs::create_dir(path)?;
    }

    Ok(())
}

pub fn create_folder(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir(path)?;
    }

    Ok(())
}

pub fn remove_folder(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(path)?;
    }

    Ok(())
}

pub fn copy_folder(from: &Path, to: &Path) -> Result<()> {
    if from.exists() {
        fs_extra::dir::copy(from, to, &fs_extra::dir::CopyOptions::new())?;
    }

    Ok(())
}

pub fn move_file(from: &Path, to: &Path) -> Result<()> {
    if from.exists() && from.is_file() {
        fs_extra::file::move_file(from, to, &fs_extra::file::CopyOptions::new())?;
    }

    Ok(())
}

pub enum DbInstance {
    Root,
    Admin,
}

pub fn add_migration_config_file_with_db_name(
    path: &Path,
    db_instance: DbInstance,
    db: &str,
) -> Result<()> {
    let username = match db_instance {
        DbInstance::Root => "root",
        DbInstance::Admin => "admin",
    };
    let password = match db_instance {
        DbInstance::Root => "root",
        DbInstance::Admin => "admin",
    };
    let port = match db_instance {
        DbInstance::Root => "8000",
        DbInstance::Admin => "8001",
    };

    let content = format!(
        r#"[core]
    schema = "less"

[db]
    address = "ws://localhost:{port}"
    username = "{username}"
    password = "{password}"
    ns = "test"
    db = "{db}""#
    );

    fs::write(path.join(".surrealdb"), content)?;

    Ok(())
}
pub fn add_migration_config_file_with_db_name_in_dir(
    path: &Path,
    db_instance: DbInstance,
    db: &str,
) -> Result<()> {
    let content_path = path.join(".surrealdb");
    let displayed_path = path.display();

    let username = match db_instance {
        DbInstance::Root => "root",
        DbInstance::Admin => "admin",
    };
    let password = match db_instance {
        DbInstance::Root => "root",
        DbInstance::Admin => "admin",
    };
    let port = match db_instance {
        DbInstance::Root => "8000",
        DbInstance::Admin => "8001",
    };

    let content = format!(
        r#"[core]
    path = "{displayed_path}"
    schema = "less"

[db]
    address = "ws://localhost:{port}"
    username = "{username}"
    password = "{password}"
    ns = "test"
    db = "{db}""#
    );

    fs::write(content_path, content)?;

    Ok(())
}
pub fn add_migration_config_file_with_core_schema(path: &Path, schema: &str) -> Result<()> {
    let content = format!(
        r#"[core]
    schema = "{schema}"

[db]
    address = "ws://localhost:8000"
    username = "root"
    password = "root"
    ns = "test"
    db = "test""#
    );

    fs::write(path.join(".surrealdb"), content)?;

    Ok(())
}
pub fn add_migration_config_file_with_db_address(path: &Path, address: &str) -> Result<()> {
    let content = format!(
        r#"[core]
    schema = "less"

[db]
    address = "{address}"
    username = "root"
    password = "root"
    ns = "test"
    db = "test""#
    );

    fs::write(path.join(".surrealdb"), content)?;

    Ok(())
}

pub fn add_simple_migration_file(path: &Path) -> Result<()> {
    let content = "DEFINE PARAM $token VALUE 'xxxxxxxx';";

    let mut cmd = create_cmd(path)?;
    cmd.arg("create")
        .arg("migration")
        .arg("AddTokenParam")
        .arg("--content")
        .arg(content)
        .arg("--down");
    cmd.assert().try_success()?;

    Ok(())
}

pub fn write_simple_migration_down_file(path: &Path, migration_name: &str) -> Result<()> {
    let content = "REMOVE PARAM $token;";
    let migration_down_file = path
        .join("migrations")
        .join("down")
        .join(format!("{migration_name}.surql"));

    fs::write(migration_down_file, content)?;

    Ok(())
}

pub fn add_post_migration_file(path: &Path) -> Result<()> {
    let content = "CREATE post SET title = 'Hello world!', content = 'This is my first post!', author = user:admin;";

    let mut cmd = create_cmd(path)?;
    cmd.arg("create")
        .arg("migration")
        .arg("AddPost")
        .arg("--content")
        .arg(content)
        .arg("--down");
    cmd.assert().try_success()?;

    Ok(())
}

pub fn write_post_migration_down_file(path: &Path, migration_name: &str) -> Result<()> {
    let content = "DELETE post;";
    let migration_down_file = path
        .join("migrations")
        .join("down")
        .join(format!("{migration_name}.surql"));

    fs::write(migration_down_file, content)?;

    Ok(())
}

pub fn add_category_schema_file(path: &Path) -> Result<()> {
    let schemas_files_dir = path.join("schemas");

    if schemas_files_dir.exists() {
        let schema_file = schemas_files_dir.join("category.surql");
        const CONTENT: &str = "DEFINE TABLE OVERWRITE category SCHEMALESS;

DEFINE FIELD OVERWRITE name ON category TYPE string;
DEFINE FIELD OVERWRITE created_at ON category TYPE datetime VALUE time::now() READONLY;";

        fs::write(schema_file, CONTENT)?;
    }

    Ok(())
}

pub fn add_invalid_schema_file(path: &Path) -> Result<()> {
    let schemas_files_dir = path.join("schemas");

    if schemas_files_dir.exists() {
        let schema_file = schemas_files_dir.join("table.surql");
        const CONTENT: &str = "DEFINE TABLE table SCHEMANONE;";

        fs::write(schema_file, CONTENT)?;
    }

    Ok(())
}

pub fn add_category_migration_file(path: &Path) -> Result<()> {
    let content = "CREATE category SET name = 'Technology';
CREATE category SET name = 'Marketing';
CREATE category SET name = 'Books';";

    let mut cmd = create_cmd(path)?;
    cmd.arg("create")
        .arg("migration")
        .arg("AddCategories")
        .arg("--content")
        .arg(content)
        .arg("--down");
    cmd.assert().try_success()?;

    Ok(())
}

pub fn write_category_migration_down_file(path: &Path, migration_name: &str) -> Result<()> {
    let content = "DELETE category;";
    let migration_down_file = path
        .join("migrations")
        .join("down")
        .join(format!("{migration_name}.surql"));

    fs::write(migration_down_file, content)?;

    Ok(())
}

pub fn add_archive_schema_file(path: &Path) -> Result<()> {
    let schemas_files_dir = path.join("schemas");

    if schemas_files_dir.exists() {
        let schema_file = schemas_files_dir.join("archive.surql");
        const CONTENT: &str = "DEFINE TABLE OVERWRITE archive SCHEMALESS;

DEFINE FIELD OVERWRITE name ON archive TYPE string;
DEFINE FIELD OVERWRITE from_date ON archive TYPE datetime;
DEFINE FIELD OVERWRITE to_date ON archive TYPE datetime;
DEFINE FIELD OVERWRITE created_at ON archive TYPE datetime VALUE time::now() READONLY;";

        fs::write(schema_file, CONTENT)?;
    }

    Ok(())
}

pub fn add_archive_migration_file(path: &Path) -> Result<()> {
    let content =
        "CREATE archive SET name = '2022', from_date = d'2022-01-01T00:00:00Z', to_date = d'2022-12-31T00:00:00Z';";

    let mut cmd = create_cmd(path)?;
    cmd.arg("create")
        .arg("migration")
        .arg("AddArchive")
        .arg("--content")
        .arg(content)
        .arg("--down");
    cmd.assert().try_success()?;

    Ok(())
}

pub fn write_archive_migration_down_file(path: &Path, migration_name: &str) -> Result<()> {
    let content = "DELETE archive;";
    let migration_down_file = path
        .join("migrations")
        .join("down")
        .join(format!("{migration_name}.surql"));

    fs::write(migration_down_file, content)?;

    Ok(())
}

pub fn add_jwks_schema_file(path: &Path) -> Result<()> {
    let schemas_files_dir = path.join("schemas");

    if schemas_files_dir.exists() {
        let schema_file = schemas_files_dir.join("jwks.surql");
        const CONTENT: &str = "DEFINE TOKEN OVERWRITE token_name
-- Use this token provider for database authorization
ON DATABASE
-- Specify the JWKS specification used to verify the token
TYPE JWKS
-- Specify the URL where the JWKS object can be found
VALUE \"https://example.com/.well-known/jwks.json\";";

        fs::write(schema_file, CONTENT)?;
    }

    Ok(())
}

pub fn add_computed_schema_file(path: &Path) -> Result<()> {
    let schemas_files_dir = path.join("schemas");

    if schemas_files_dir.exists() {
        let schema_file = schemas_files_dir.join("computed.surql");
        const CONTENT: &str = "DEFINE TABLE OVERWRITE computed AS
    SELECT title, content, status, author
    FROM post
    GROUP BY author;";

        fs::write(schema_file, CONTENT)?;
    }

    Ok(())
}

pub fn inline_down_migration_files(path: &Path) -> Result<()> {
    let migrations_files_dir = path.join("migrations");
    let down_migrations_files_dir = migrations_files_dir.join("down");

    let down_migrations_files = down_migrations_files_dir
        .read_dir()?
        .filter(|entry| match entry.as_ref() {
            Ok(entry) => entry.path().is_file(),
            Err(_) => false,
        })
        .collect::<Vec<_>>();

    for down_migrations_file in down_migrations_files {
        let down_migration_file = down_migrations_file?;
        let down_migration_file_name = down_migration_file.file_name();
        let down_migration_file_name = down_migration_file_name
            .to_str()
            .context("Invalid file name")?;
        let down_migration_file_name = down_migration_file_name.replace(".surql", "");

        let inlined_down_migration_file_name = migrations_files_dir.join(down_migration_file_name);
        let inlined_down_migration_file_name =
            inlined_down_migration_file_name.with_extension("down.surql");

        let down_migration_file_content = fs::read_to_string(down_migration_file.path())?;

        fs::write(
            inlined_down_migration_file_name,
            down_migration_file_content,
        )?;
    }

    remove_folder(&down_migrations_files_dir)?;

    Ok(())
}

pub fn disable_checksum_capability(path: &Path) -> Result<()> {
    let schemas_files_dir = path.join("schemas");

    if schemas_files_dir.exists() {
        let schema_file = schemas_files_dir.join("script_migration.surql");
        let content = fs::read(&schema_file)?;
        let content = String::from_utf8(content)?;
        let content = content.replace(
            "DEFINE FIELD OVERWRITE checksum ON script_migration TYPE option<string>;",
            "",
        );

        fs::write(schema_file, content)?;
    }

    Ok(())
}
