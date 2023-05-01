use anyhow::{anyhow, Context, Result};
use convert_case::{Case, Casing};
use include_dir::{include_dir, Dir};
use std::{
    collections::HashMap,
    io::Write,
    ops::Deref,
    path::{Path, PathBuf},
};

use crate::{
    cli::{ScaffoldSchemaDbType, ScaffoldTemplate},
    config,
    constants::{EVENTS_DIR_NAME, MIGRATIONS_DIR_NAME, SCHEMAS_DIR_NAME},
};

pub fn from_template(template: ScaffoldTemplate) -> Result<()> {
    let folder_path = config::retrieve_folder_path();

    apply_before_scaffold(folder_path.to_owned())?;

    copy_template_files_to_current_dir(template, folder_path.to_owned())?;

    apply_after_scaffold(folder_path.to_owned())?;

    Ok(())
}

pub fn from_schema(
    schema: String,
    db_type: ScaffoldSchemaDbType,
    preserve_casing: bool,
) -> Result<()> {
    let folder_path = config::retrieve_folder_path();

    apply_before_scaffold(folder_path.to_owned())?;

    scaffold_from_schema(schema, db_type, preserve_casing, folder_path.to_owned())?;

    apply_after_scaffold(folder_path.to_owned())?;

    Ok(())
}

fn apply_before_scaffold(folder_path: Option<String>) -> Result<()> {
    let schemas_dir_path = concat_path(&folder_path, SCHEMAS_DIR_NAME);
    let events_dir_path = concat_path(&folder_path, EVENTS_DIR_NAME);
    let migrations_dir_path = concat_path(&folder_path, MIGRATIONS_DIR_NAME);

    fails_if_folder_already_exists(&schemas_dir_path, SCHEMAS_DIR_NAME)?;
    fails_if_folder_already_exists(&events_dir_path, EVENTS_DIR_NAME)?;
    fails_if_folder_already_exists(&migrations_dir_path, MIGRATIONS_DIR_NAME)?;

    Ok(())
}

fn apply_after_scaffold(folder_path: Option<String>) -> Result<()> {
    let schemas_dir_path = concat_path(&folder_path, SCHEMAS_DIR_NAME);
    let events_dir_path = concat_path(&folder_path, EVENTS_DIR_NAME);
    let migrations_dir_path = concat_path(&folder_path, MIGRATIONS_DIR_NAME);

    ensures_folder_exists(&schemas_dir_path)?;
    ensures_folder_exists(&events_dir_path)?;
    ensures_folder_exists(&migrations_dir_path)?;

    rename_migrations_files_to_match_current_date(&migrations_dir_path)?;

    Ok(())
}

fn concat_path(folder_path: &Option<String>, dir_name: &str) -> PathBuf {
    match folder_path.to_owned() {
        Some(folder_path) => Path::new(&folder_path).join(dir_name),
        None => Path::new(dir_name).to_path_buf(),
    }
}

fn fails_if_folder_already_exists(dir_path: &PathBuf, dir_name: &str) -> Result<()> {
    match dir_path.exists() {
        true => Err(anyhow!("'{}' folder already exists.", dir_name)),
        false => Ok(()),
    }
}

fn copy_template_files_to_current_dir(
    template: ScaffoldTemplate,
    folder_path: Option<String>,
) -> Result<()> {
    const TEMPLATES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates");

    let template_dir_name = get_template_name(template);
    let from = TEMPLATES_DIR
        .get_dir(template_dir_name)
        .context("Cannot get template dir")?;

    let to = match folder_path.to_owned() {
        Some(folder_path) => folder_path,
        None => ".".to_owned(),
    };

    extract(from, to)?;

    Ok(())
}

fn get_template_name(template: ScaffoldTemplate) -> &'static str {
    match template {
        ScaffoldTemplate::Empty => "empty",
        ScaffoldTemplate::Blog => "blog",
        ScaffoldTemplate::Ecommerce => "ecommerce",
    }
}

// 💡 Function extract customized because it is not implemented in the "include_dir" crate.
// cf. https://github.com/Michael-F-Bryan/include_dir/pull/60
pub fn extract<S: AsRef<Path>>(dir: &Dir<'_>, path: S) -> std::io::Result<()> {
    fn extract_dir<S: AsRef<Path>>(dir: Dir<'_>, path: S) -> std::io::Result<()> {
        let path = path.as_ref();

        for dir in dir.dirs() {
            let dir_path = dir.path().components().skip(1).collect::<PathBuf>();

            std::fs::create_dir_all(path.join(dir_path))?;
            extract_dir(dir.clone(), path)?;
        }

        for file in dir.files() {
            let file_path = file.path().components().skip(1).collect::<PathBuf>();

            let mut fsf = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path.join(file_path))?;
            fsf.write_all(file.contents())?;
            fsf.sync_all()?;
        }

        Ok(())
    }

    extract_dir(dir.clone(), path)
}

fn scaffold_from_schema(
    schema: String,
    db_type: ScaffoldSchemaDbType,
    preserve_casing: bool,
    folder_path: Option<String>,
) -> Result<()> {
    let schema_content = std::fs::read_to_string(schema)?;

    let dialect = get_sql_dialect(db_type);

    let ast = sqlparser::parser::Parser::parse_sql(dialect.deref(), schema_content.as_str())?;

    let schema = convert_ast_to_surrealdb_schema(ast, preserve_casing)?;

    if schema.tables.len() <= 0 {
        return Err(anyhow!("No table found in schema file."));
    }

    if schema.tables.contains_key("script_migration") {
        return Err(anyhow!(
            "The table 'script_migration' is reserved for internal use."
        ));
    }

    copy_template_files_to_current_dir(ScaffoldTemplate::Empty, folder_path.to_owned())?;

    let schemas_dir_path = concat_path(&folder_path, SCHEMAS_DIR_NAME);

    for (table_name, table_definition) in schema.tables {
        let filename = format!("{}.surql", table_name);

        let path = schemas_dir_path.join(filename);
        std::fs::write(path, table_definition)?;
    }

    Ok(())
}

fn get_sql_dialect(db_type: ScaffoldSchemaDbType) -> Box<dyn sqlparser::dialect::Dialect> {
    match db_type {
        ScaffoldSchemaDbType::BigQuery => Box::new(sqlparser::dialect::BigQueryDialect {}),
        ScaffoldSchemaDbType::ClickHouse => Box::new(sqlparser::dialect::ClickHouseDialect {}),
        ScaffoldSchemaDbType::Hive => Box::new(sqlparser::dialect::HiveDialect {}),
        ScaffoldSchemaDbType::MsSql => Box::new(sqlparser::dialect::MsSqlDialect {}),
        ScaffoldSchemaDbType::MySql => Box::new(sqlparser::dialect::MySqlDialect {}),
        ScaffoldSchemaDbType::PostgreSql => Box::new(sqlparser::dialect::PostgreSqlDialect {}),
        ScaffoldSchemaDbType::Redshift => Box::new(sqlparser::dialect::RedshiftSqlDialect {}),
        ScaffoldSchemaDbType::SQLite => Box::new(sqlparser::dialect::SQLiteDialect {}),
        ScaffoldSchemaDbType::Snowflake => Box::new(sqlparser::dialect::SnowflakeDialect {}),
    }
}

type SurrealdbSchemaName = String;
type SurrealdbSchemaDefinition = String;
type SurrealdbSchemaTable = HashMap<SurrealdbSchemaName, SurrealdbSchemaDefinition>;

#[derive(Debug)]
struct SurrealdbSchema {
    tables: SurrealdbSchemaTable,
}

fn convert_ast_to_surrealdb_schema(
    ast: Vec<sqlparser::ast::Statement>,
    preserve_casing: bool,
) -> Result<SurrealdbSchema> {
    let mut tables = SurrealdbSchemaTable::new();

    for statement in ast {
        match statement {
            sqlparser::ast::Statement::CreateTable { name, columns, .. } => {
                let mut definition = SurrealdbSchemaDefinition::new();

                let table_name = name.to_string();
                let table_name = match preserve_casing {
                    true => table_name,
                    false => table_name.to_case(Case::Snake),
                };

                definition.push_str(&format!("DEFINE TABLE {} SCHEMALESS;\n\n", table_name));

                for column in columns {
                    let column_name = column.name.value.to_string();
                    let column_name = match preserve_casing {
                        true => column_name,
                        false => column_name.to_case(Case::Snake),
                    };

                    let column_type = detect_column_type(column);

                    let type_definition = match column_type {
                        Some(column_type) => format!(" TYPE {}", column_type),
                        None => String::new(),
                    };

                    definition.push_str(&format!(
                        "DEFINE FIELD {} ON {}{};\n",
                        column_name, table_name, type_definition
                    ));
                }

                tables.insert(table_name, definition);
            }
            _ => {}
        }
    }

    Ok(SurrealdbSchema { tables })
}

fn detect_column_type(column: sqlparser::ast::ColumnDef) -> Option<&'static str> {
    match column.data_type {
        sqlparser::ast::DataType::TinyInt(_) => Some("number"),
        sqlparser::ast::DataType::UnsignedTinyInt(_) => Some("number"),
        sqlparser::ast::DataType::SmallInt(_) => Some("number"),
        sqlparser::ast::DataType::UnsignedSmallInt(_) => Some("number"),
        sqlparser::ast::DataType::Int(_) => Some("number"),
        sqlparser::ast::DataType::UnsignedInt(_) => Some("number"),
        sqlparser::ast::DataType::Integer(_) => Some("number"),
        sqlparser::ast::DataType::UnsignedInteger(_) => Some("number"),
        sqlparser::ast::DataType::MediumInt(_) => Some("number"),
        sqlparser::ast::DataType::UnsignedMediumInt(_) => Some("number"),
        sqlparser::ast::DataType::BigInt(_) => Some("number"),
        sqlparser::ast::DataType::UnsignedBigInt(_) => Some("number"),
        sqlparser::ast::DataType::Real => Some("number"),
        sqlparser::ast::DataType::Double => Some("number"),
        sqlparser::ast::DataType::DoublePrecision => Some("number"),
        sqlparser::ast::DataType::Dec { .. } => Some("number"),
        sqlparser::ast::DataType::Decimal { .. } => Some("number"),
        sqlparser::ast::DataType::BigDecimal(_) => Some("number"),
        sqlparser::ast::DataType::Float { .. } => Some("number"),
        sqlparser::ast::DataType::Numeric(_) => Some("number"),
        sqlparser::ast::DataType::BigNumeric(_) => Some("number"),
        sqlparser::ast::DataType::Char { .. } => Some("string"),
        sqlparser::ast::DataType::CharVarying { .. } => Some("string"),
        sqlparser::ast::DataType::Character { .. } => Some("string"),
        sqlparser::ast::DataType::CharacterVarying { .. } => Some("string"),
        sqlparser::ast::DataType::Varchar { .. } => Some("string"),
        sqlparser::ast::DataType::Nvarchar(_) => Some("string"),
        sqlparser::ast::DataType::Text => Some("string"),
        sqlparser::ast::DataType::Boolean => Some("bool"),
        sqlparser::ast::DataType::Date => Some("datetime"),
        sqlparser::ast::DataType::Time { .. } => Some("datetime"),
        sqlparser::ast::DataType::Datetime(_) => Some("datetime"),
        sqlparser::ast::DataType::Timestamp { .. } => Some("datetime"),
        sqlparser::ast::DataType::Interval { .. } => Some("duration"),
        sqlparser::ast::DataType::JSON => Some("object"),
        sqlparser::ast::DataType::Array(_) => Some("array"),
        sqlparser::ast::DataType::Custom(sqlparser::ast::ObjectName(identifiers), _) => {
            if let Some(first_identifier) = identifiers.first() {
                // 💡 MSSQL type for boolean
                if first_identifier.value.to_string() == "BIT" {
                    Some("bool")
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

fn ensures_folder_exists(dir_path: &PathBuf) -> Result<()> {
    if !dir_path.exists() {
        fs_extra::dir::create_all(&dir_path, false)?;
    }

    Ok(())
}

fn rename_migrations_files_to_match_current_date(migrations_dir_path: &PathBuf) -> Result<()> {
    let now = chrono::Local::now();
    let regex = regex::Regex::new(r"^YYYYMMDD_HHMM(\d{2})_")?;

    let migrations_dir = std::fs::read_dir(&migrations_dir_path)?;

    let migration_filenames_to_rename = migrations_dir
        .filter_map(|entry| match entry {
            Ok(file) => {
                let file_name = file.file_name().to_owned();
                if regex.is_match(file_name.to_str().unwrap_or("")) {
                    Some(file_name)
                } else {
                    None
                }
            }
            Err(_) => None,
        })
        .collect::<Vec<_>>();

    for filename in migration_filenames_to_rename {
        let filename = filename
            .to_str()
            .context("Cannot convert filename to string")?;

        let captures = regex
            .captures(filename)
            .context("Cannot retrieve from pattern")?;
        let seconds = captures
            .get(1)
            .context("Cannot retrieve from pattern")?
            .as_str();

        let new_filename_prefix = format!("{}{}_", now.format("%Y%m%d_%H%M"), seconds);
        let new_filename = regex.replace(filename, new_filename_prefix);

        let from = format!("{}/{}", migrations_dir_path.display(), filename);
        let to = format!("{}/{}", migrations_dir_path.display(), new_filename);

        std::fs::rename(from, to)?;
    }

    Ok(())
}
