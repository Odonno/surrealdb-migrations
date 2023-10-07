use anyhow::{anyhow, Result};
use convert_case::{Case, Casing};
use std::{collections::HashMap, ops::Deref};

use crate::{
    cli::{ScaffoldSchemaDbType, ScaffoldTemplate},
    config,
    constants::{SCHEMAS_DIR_NAME, SCRIPT_MIGRATION_TABLE_NAME},
    io,
};

use super::common::{
    apply_after_scaffold, apply_before_scaffold, copy_template_files_to_current_dir,
};

pub struct ScaffoldFromSchemaArgs<'a> {
    pub schema: String,
    pub db_type: ScaffoldSchemaDbType,
    pub preserve_casing: bool,
    pub config_file: Option<&'a str>,
}

pub fn main(args: ScaffoldFromSchemaArgs) -> Result<()> {
    let ScaffoldFromSchemaArgs {
        schema,
        db_type,
        preserve_casing,
        config_file,
    } = args;

    let folder_path = config::retrieve_folder_path(config_file);

    apply_before_scaffold(folder_path.to_owned())?;

    scaffold_from_schema(schema, db_type, preserve_casing, folder_path.to_owned())?;

    apply_after_scaffold(folder_path)?;

    Ok(())
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
    not_null: bool,
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

    if schema.tables.is_empty() {
        return Err(anyhow!("No table found in schema file."));
    }

    if schema.tables.contains_key(SCRIPT_MIGRATION_TABLE_NAME) {
        return Err(anyhow!(
            "The table '{}' is reserved for internal use.",
            SCRIPT_MIGRATION_TABLE_NAME
        ));
    }

    copy_template_files_to_current_dir(ScaffoldTemplate::Empty, folder_path.to_owned())?;

    let schemas_dir_path = io::concat_path(&folder_path, SCHEMAS_DIR_NAME);

    for (table_name, line_definitions) in schema.tables {
        let filename = format!("{}.surql", table_name);

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

                    let assert_str = if field_definition.not_null {
                        " ASSERT $value != NONE"
                    } else {
                        ""
                    };

                    table_definition_str.push_str(&format!(
                        "DEFINE FIELD {} ON {}{}{};\n",
                        field_definition.name, table_name, field_type_str, assert_str
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
                        if let sqlparser::ast::TableConstraint::ForeignKey {
                            columns,
                            foreign_table,
                            referred_columns,
                            ..
                        } = constraint
                        {
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
                    }

                    let mut is_not_null = false;

                    // Detect unique constraints
                    for column_option in &column.options {
                        if column_option.option == sqlparser::ast::ColumnOption::NotNull {
                            is_not_null = true;
                        }
                    }

                    let line_definition =
                        SurrealdbSchemaLineDefinition::Field(SurrealdbSchemaFieldDefinition {
                            name: field_name.to_string(),
                            type_: field_type,
                            not_null: is_not_null,
                        });
                    line_definitions.push(line_definition);

                    // Detect unique constraints
                    for column_option in &column.options {
                        let option_name = &column_option.name;
                        let index_name = match option_name {
                            Some(name) => name.value.to_string(),
                            None => format!("{}_{}_index", table_name, field_name),
                        };

                        if let sqlparser::ast::ColumnOption::Unique { is_primary } =
                            column_option.option
                        {
                            if !is_primary {
                                let line_definition: SurrealdbSchemaLineDefinition =
                                    SurrealdbSchemaLineDefinition::Index(
                                        SurrealdbSchemaIndexDefinition {
                                            name: index_name,
                                            field_names: vec![field_name.to_string()],
                                            unique: true,
                                        },
                                    );
                                line_definitions.push(line_definition);
                            }
                        }
                    }
                }

                tables.insert(table_name, line_definitions);
            }
            sqlparser::ast::Statement::CreateIndex {
                name,
                table_name,
                columns,
                unique,
                ..
            } => {
                let table_name = match table_name.0.first() {
                    Some(table_name) => table_name.value.to_string(),
                    None => {
                        continue;
                    }
                };
                let table_name = match preserve_casing {
                    true => table_name,
                    false => table_name.to_case(Case::Snake),
                };

                let field_names = columns
                    .iter()
                    .map(|c| match &c.expr {
                        sqlparser::ast::Expr::Identifier(ident) => ident.value.to_string(),
                        _ => {
                            panic!("Only identifier expressions are supported for index columns");
                        }
                    })
                    .map(|name| match preserve_casing {
                        true => name,
                        false => name.to_case(Case::Snake),
                    })
                    .collect::<Vec<_>>();

                let index_name = match name {
                    Some(name) => match name.0.first() {
                        Some(identifier) => identifier.value.to_string(),
                        None => format!("{}_{}_index", table_name, field_names.join("_")),
                    },
                    None => format!("{}_{}_index", table_name, field_names.join("_")),
                };

                let line_definition: SurrealdbSchemaLineDefinition =
                    SurrealdbSchemaLineDefinition::Index(SurrealdbSchemaIndexDefinition {
                        name: index_name,
                        field_names,
                        unique,
                    });

                let line_definitions = tables
                    .entry(table_name)
                    .or_insert(SurrealdbSchemaDefinition::new());
                line_definitions.push(line_definition);
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
                if first_identifier.value == "BIT" {
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
