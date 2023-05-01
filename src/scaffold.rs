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

#[derive(Debug)]
enum SurrealdbFieldType {
    Number,
    String,
    Boolean,
    DateTime,
    Duration,
    Object,
    Array,
    Record(Vec<String>),
}

#[derive(Debug)]
struct SurrealdbSchemaFieldDefinition {
    name: String,
    type_: Option<SurrealdbFieldType>,
}

#[derive(Debug)]
struct SurrealdbSchemaIndexDefinition {
    name: String,
    field_names: Vec<String>,
    unique: bool,
}

#[derive(Debug)]
enum SurrealdbSchemaLineDefinition {
    Field(SurrealdbSchemaFieldDefinition),
    Index(SurrealdbSchemaIndexDefinition),
}

type SurrealdbSchemaName = String;
type SurrealdbSchemaDefinition = Vec<SurrealdbSchemaLineDefinition>;
type SurrealdbSchemaTable = HashMap<SurrealdbSchemaName, SurrealdbSchemaDefinition>;

#[derive(Debug)]
struct SurrealdbSchema {
    tables: SurrealdbSchemaTable,
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

    for (table_name, line_definitions) in schema.tables {
        let filename = format!("{}.surql", table_name);

        println!("{:?}", line_definitions);

        let mut table_definition_str = String::new();

        table_definition_str.push_str(&format!("DEFINE TABLE {} SCHEMALESS;\n\n", table_name));

        for line_definition in line_definitions {
            match line_definition {
                SurrealdbSchemaLineDefinition::Field(field_definition) => {
                    let field_type_str = match field_definition.type_ {
                        Some(field_type) => {
                            let display_type = match field_type {
                                SurrealdbFieldType::Number => "number".to_string(),
                                SurrealdbFieldType::String => "string".to_string(),
                                SurrealdbFieldType::Boolean => "bool".to_string(),
                                SurrealdbFieldType::DateTime => "datetime".to_string(),
                                SurrealdbFieldType::Duration => "duration".to_string(),
                                SurrealdbFieldType::Object => "object".to_string(),
                                SurrealdbFieldType::Array => "array".to_string(),
                                SurrealdbFieldType::Record(tables) => {
                                    format!("record({})", tables.join(", "))
                                }
                            };
                            format!(" TYPE {}", display_type)
                        }
                        None => String::new(),
                    };

                    table_definition_str.push_str(&format!(
                        "DEFINE FIELD {} ON {}{};\n",
                        field_definition.name, table_name, field_type_str
                    ));
                }
                SurrealdbSchemaLineDefinition::Index(index_definition) => {
                    let suffix = if index_definition.unique {
                        " UNIQUE"
                    } else {
                        ""
                    };

                    table_definition_str.push_str(&format!(
                        "DEFINE INDEX {} ON {} COLUMNS {}{};\n",
                        index_definition.name,
                        table_name,
                        index_definition.field_names.join(", "),
                        suffix
                    ));
                }
            }
        }

        let path = schemas_dir_path.join(filename);
        std::fs::write(path, table_definition_str)?;
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

fn convert_ast_to_surrealdb_schema(
    ast: Vec<sqlparser::ast::Statement>,
    preserve_casing: bool,
) -> Result<SurrealdbSchema> {
    let mut tables = SurrealdbSchemaTable::new();

    for statement in ast {
        match statement {
            sqlparser::ast::Statement::CreateTable {
                name,
                columns,
                constraints,
                ..
            } => {
                let mut line_definitions = SurrealdbSchemaDefinition::new();

                let table_name = name.to_string();
                let table_name = match preserve_casing {
                    true => table_name,
                    false => table_name.to_case(Case::Snake),
                };

                for column in columns {
                    let field_name = column.name.value.to_string();
                    let field_name = match preserve_casing {
                        true => field_name,
                        false => field_name.to_case(Case::Snake),
                    };

                    let mut field_type = detect_field_type(&column);

                    // Detect record type from foreign key (if any)
                    for constraint in &constraints {
                        match constraint {
                            sqlparser::ast::TableConstraint::ForeignKey {
                                columns,
                                foreign_table,
                                referred_columns,
                                ..
                            } => {
                                if columns.len() != 1 {
                                    continue;
                                }

                                if referred_columns.len() != 1 {
                                    continue;
                                }

                                let column_identifier = columns.first().unwrap().value.to_string();
                                let column_identifier = match preserve_casing {
                                    true => column_identifier,
                                    false => column_identifier.to_case(Case::Snake),
                                };

                                if field_name != column_identifier {
                                    continue;
                                }

                                let referred_column =
                                    referred_columns.first().unwrap().value.to_string();

                                if referred_column.to_lowercase() != "id" {
                                    continue;
                                }

                                let foreign_table = foreign_table.0.first();
                                if foreign_table.is_none() {
                                    continue;
                                }

                                let foreign_table = foreign_table.unwrap().value.to_string();
                                let foreign_table = match preserve_casing {
                                    true => foreign_table,
                                    false => foreign_table.to_case(Case::Snake),
                                };

                                field_type = match field_type {
                                    Some(SurrealdbFieldType::Record(tables)) => {
                                        Some(SurrealdbFieldType::Record(
                                            tables.into_iter().chain(vec![foreign_table]).collect(),
                                        ))
                                    }
                                    _ => Some(SurrealdbFieldType::Record(vec![foreign_table])),
                                };
                            }
                            _ => {}
                        }
                    }

                    let line_definition =
                        SurrealdbSchemaLineDefinition::Field(SurrealdbSchemaFieldDefinition {
                            name: field_name.to_string(),
                            type_: field_type,
                        });
                    line_definitions.push(line_definition);

                    // Detect unique constraints
                    for column_option in &column.options {
                        let option_name = &column_option.name;
                        let option_name = match option_name {
                            Some(name) => name.value.to_string(),
                            None => format!("{}_{}_index", table_name, field_name.to_string()),
                        };

                        match column_option.option {
                            sqlparser::ast::ColumnOption::Unique { is_primary } => {
                                if !is_primary {
                                    let line_definition: SurrealdbSchemaLineDefinition =
                                        SurrealdbSchemaLineDefinition::Index(
                                            SurrealdbSchemaIndexDefinition {
                                                name: option_name,
                                                field_names: vec![field_name.to_string()],
                                                unique: true,
                                            },
                                        );
                                    line_definitions.push(line_definition);
                                }
                            }
                            _ => {}
                        }
                    }
                }

                tables.insert(table_name, line_definitions);
            }
            _ => {}
        }
    }

    Ok(SurrealdbSchema { tables })
}

fn detect_field_type(column: &sqlparser::ast::ColumnDef) -> Option<SurrealdbFieldType> {
    match &column.data_type {
        sqlparser::ast::DataType::TinyInt(_) => Some(SurrealdbFieldType::Number),
        sqlparser::ast::DataType::UnsignedTinyInt(_) => Some(SurrealdbFieldType::Number),
        sqlparser::ast::DataType::SmallInt(_) => Some(SurrealdbFieldType::Number),
        sqlparser::ast::DataType::UnsignedSmallInt(_) => Some(SurrealdbFieldType::Number),
        sqlparser::ast::DataType::Int(_) => Some(SurrealdbFieldType::Number),
        sqlparser::ast::DataType::UnsignedInt(_) => Some(SurrealdbFieldType::Number),
        sqlparser::ast::DataType::Integer(_) => Some(SurrealdbFieldType::Number),
        sqlparser::ast::DataType::UnsignedInteger(_) => Some(SurrealdbFieldType::Number),
        sqlparser::ast::DataType::MediumInt(_) => Some(SurrealdbFieldType::Number),
        sqlparser::ast::DataType::UnsignedMediumInt(_) => Some(SurrealdbFieldType::Number),
        sqlparser::ast::DataType::BigInt(_) => Some(SurrealdbFieldType::Number),
        sqlparser::ast::DataType::UnsignedBigInt(_) => Some(SurrealdbFieldType::Number),
        sqlparser::ast::DataType::Real => Some(SurrealdbFieldType::Number),
        sqlparser::ast::DataType::Double => Some(SurrealdbFieldType::Number),
        sqlparser::ast::DataType::DoublePrecision => Some(SurrealdbFieldType::Number),
        sqlparser::ast::DataType::Dec { .. } => Some(SurrealdbFieldType::Number),
        sqlparser::ast::DataType::Decimal { .. } => Some(SurrealdbFieldType::Number),
        sqlparser::ast::DataType::BigDecimal(_) => Some(SurrealdbFieldType::Number),
        sqlparser::ast::DataType::Float { .. } => Some(SurrealdbFieldType::Number),
        sqlparser::ast::DataType::Numeric(_) => Some(SurrealdbFieldType::Number),
        sqlparser::ast::DataType::BigNumeric(_) => Some(SurrealdbFieldType::Number),
        sqlparser::ast::DataType::Char { .. } => Some(SurrealdbFieldType::String),
        sqlparser::ast::DataType::CharVarying { .. } => Some(SurrealdbFieldType::String),
        sqlparser::ast::DataType::Character { .. } => Some(SurrealdbFieldType::String),
        sqlparser::ast::DataType::CharacterVarying { .. } => Some(SurrealdbFieldType::String),
        sqlparser::ast::DataType::Varchar { .. } => Some(SurrealdbFieldType::String),
        sqlparser::ast::DataType::Nvarchar(_) => Some(SurrealdbFieldType::String),
        sqlparser::ast::DataType::Text => Some(SurrealdbFieldType::String),
        sqlparser::ast::DataType::Boolean => Some(SurrealdbFieldType::Boolean),
        sqlparser::ast::DataType::Date => Some(SurrealdbFieldType::DateTime),
        sqlparser::ast::DataType::Time { .. } => Some(SurrealdbFieldType::DateTime),
        sqlparser::ast::DataType::Datetime(_) => Some(SurrealdbFieldType::DateTime),
        sqlparser::ast::DataType::Timestamp { .. } => Some(SurrealdbFieldType::DateTime),
        sqlparser::ast::DataType::Interval { .. } => Some(SurrealdbFieldType::Duration),
        sqlparser::ast::DataType::JSON => Some(SurrealdbFieldType::Object),
        sqlparser::ast::DataType::Array(_) => Some(SurrealdbFieldType::Array),
        sqlparser::ast::DataType::Custom(sqlparser::ast::ObjectName(identifiers), _) => {
            if let Some(first_identifier) = identifiers.first() {
                // 💡 MSSQL type for boolean
                if first_identifier.value.to_string() == "BIT" {
                    Some(SurrealdbFieldType::Boolean)
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
