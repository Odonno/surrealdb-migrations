use std::path::Path;

use anyhow::{ensure, Result};
use serial_test::serial;

use crate::helpers::*;

#[test]
#[serial]
fn replay_migrations_on_clean_db() -> Result<()> {
    clear_tests_files()?;
    scaffold_blog_template()?;

    run_with_surreal_instance(|| {
        apply_migrations()?;

        Ok(())
    })?;

    run_with_surreal_instance(|| {
        let mut cmd = create_cmd()?;

        cmd.arg("apply");

        cmd.assert().try_success().and_then(|assert| {
            assert.try_stdout(
                "Executing migration AddAdminUser...
Executing migration AddPost...
Executing migration CommentPost...
Schema files successfully executed!
Event files successfully executed!
Migration files successfully executed!\n",
            )
        })?;

        let definitions_files =
            std::fs::read_dir("tests-files/migrations/definitions")?.filter(|entry| {
                match entry.as_ref() {
                    Ok(entry) => entry.path().is_file(),
                    Err(_) => false,
                }
            });
        ensure!(definitions_files.count() == 1);

        ensure!(Path::new("tests-files/migrations/definitions/_initial.json").exists());

        Ok(())
    })
}

#[tokio::test]
#[serial]
async fn apply_3_consecutives_schema_and_data_changes() -> Result<()> {
    clear_tests_files()?;
    scaffold_blog_template()?;
    empty_folder("tests-files/migrations")?;

    run_with_surreal_instance_async(|| {
        Box::pin(async {
            // First migration
            add_post_migration_file()?;

            let first_migration_name = get_first_migration_name()?;

            apply_migrations_up_to(&first_migration_name)?;

            // Check definitions files
            let definitions_files = std::fs::read_dir("tests-files/migrations/definitions")?
                .filter(|entry| match entry.as_ref() {
                    Ok(entry) => entry.path().is_file(),
                    Err(_) => false,
                });
            ensure!(definitions_files.count() == 1);

            const INITIAL_DEFINITION_FILE_PATH: &str =
                "tests-files/migrations/definitions/_initial.json";

            ensure!(Path::new(INITIAL_DEFINITION_FILE_PATH).exists());

            let initial_migration_definition_str =
                std::fs::read_to_string(INITIAL_DEFINITION_FILE_PATH)?;
            let initial_migration_definition =
                serde_json::from_str::<MigrationDefinition>(&initial_migration_definition_str)?;

            ensure!(
                initial_migration_definition.schemas
                    == Some(INITIAL_DEFINITION_SCHEMAS.to_string())
            );
            ensure!(
                initial_migration_definition.events == Some(INITIAL_DEFINITION_EVENTS.to_string())
            );

            // Check data
            let is_table_empty = is_surreal_table_empty(None, "post").await?;
            ensure!(
                !is_table_empty,
                "First migration: 'post' table should not be empty"
            );

            let is_table_empty = is_surreal_table_empty(None, "category").await?;
            ensure!(
                is_table_empty,
                "First migration: 'category' table should be empty"
            );

            let is_table_empty = is_surreal_table_empty(None, "archive").await?;
            ensure!(
                is_table_empty,
                "First migration: 'archive' table should be empty"
            );

            std::thread::sleep(std::time::Duration::from_secs(1));

            // Second migration
            add_category_schema_file()?;
            add_category_migration_file()?;

            let second_migration_name = get_second_migration_name()?;

            apply_migrations_up_to(&second_migration_name)?;

            // Check definitions files
            let definitions_files = std::fs::read_dir("tests-files/migrations/definitions")?
                .filter(|entry| match entry.as_ref() {
                    Ok(entry) => entry.path().is_file(),
                    Err(_) => false,
                });
            ensure!(definitions_files.count() == 2);

            let second_migration_definition_file_path: String = format!(
                "tests-files/migrations/definitions/{}.json",
                &second_migration_name
            );

            ensure!(Path::new(INITIAL_DEFINITION_FILE_PATH).exists());
            ensure!(Path::new(&second_migration_definition_file_path).exists());

            let new_initial_migration_definition_str =
                std::fs::read_to_string(INITIAL_DEFINITION_FILE_PATH)?;
            ensure!(
                initial_migration_definition_str == new_initial_migration_definition_str,
                "Second migration: Initial definition file should not have changed"
            );

            let second_migration_definition_str =
                std::fs::read_to_string(&second_migration_definition_file_path)?;
            let second_migration_definition =
                serde_json::from_str::<MigrationDefinition>(&second_migration_definition_str)?;

            ensure!(
                second_migration_definition.schemas == Some(SECOND_MIGRATION_SCHEMAS.to_string())
            );
            ensure!(second_migration_definition.events.is_none());

            // Check data
            let is_table_empty = is_surreal_table_empty(None, "post").await?;
            ensure!(
                !is_table_empty,
                "Second migration: 'post' table should not be empty"
            );

            let is_table_empty = is_surreal_table_empty(None, "category").await?;
            ensure!(
                !is_table_empty,
                "Second migration: 'category' table should not be empty"
            );

            let is_table_empty = is_surreal_table_empty(None, "archive").await?;
            ensure!(
                is_table_empty,
                "Second migration: 'archive' table should be empty"
            );

            std::thread::sleep(std::time::Duration::from_secs(1));

            // Last migration
            add_archive_schema_file()?;
            add_archive_migration_file()?;

            let third_migration_name = get_third_migration_name()?;

            apply_migrations()?;

            // Check definitions files
            let definitions_files = std::fs::read_dir("tests-files/migrations/definitions")?
                .filter(|entry| match entry.as_ref() {
                    Ok(entry) => entry.path().is_file(),
                    Err(_) => false,
                });
            ensure!(definitions_files.count() == 3);

            let third_migration_definition_file_path: String = format!(
                "tests-files/migrations/definitions/{}.json",
                &third_migration_name
            );

            ensure!(Path::new(INITIAL_DEFINITION_FILE_PATH).exists());
            ensure!(Path::new(&second_migration_definition_file_path).exists());
            ensure!(Path::new(&third_migration_definition_file_path).exists());

            let new_initial_migration_definition_str =
                std::fs::read_to_string(INITIAL_DEFINITION_FILE_PATH)?;
            ensure!(
                initial_migration_definition_str == new_initial_migration_definition_str,
                "Last migration: Initial definition file should not have changed"
            );

            let third_migration_definition_str =
                std::fs::read_to_string(&third_migration_definition_file_path)?;
            let third_migration_definition =
                serde_json::from_str::<MigrationDefinition>(&third_migration_definition_str)?;

            ensure!(
                third_migration_definition.schemas == Some(THIRD_MIGRATION_SCHEMAS.to_string())
            );
            ensure!(third_migration_definition.events.is_none());

            // Check data
            let is_table_empty = is_surreal_table_empty(None, "post").await?;
            ensure!(
                !is_table_empty,
                "Last migration: 'post' table should not be empty"
            );

            let is_table_empty = is_surreal_table_empty(None, "category").await?;
            ensure!(
                !is_table_empty,
                "Last migration: 'category' table should not be empty"
            );

            let is_table_empty = is_surreal_table_empty(None, "archive").await?;
            ensure!(
                !is_table_empty,
                "Last migration: 'archive' table should not be empty"
            );

            Ok(())
        })
    })
    .await
}

#[tokio::test]
#[serial]
async fn apply_3_consecutives_schema_and_data_changes_on_clean_db() -> Result<()> {
    clear_tests_files()?;
    scaffold_blog_template()?;
    empty_folder("tests-files/migrations")?;

    run_with_surreal_instance_async(|| {
        Box::pin(async {
            // First migration
            add_post_migration_file()?;

            let first_migration_name = get_first_migration_name()?;

            apply_migrations_up_to(&first_migration_name)?;

            // Check db schema
            let table_definitions = get_surrealdb_table_definitions(None).await?;
            ensure!(
                table_definitions.len() == 6,
                "First run, first migration: wrong number of tables"
            );

            std::thread::sleep(std::time::Duration::from_secs(1));

            // Second migration
            add_category_schema_file()?;
            add_category_migration_file()?;

            let second_migration_name = get_second_migration_name()?;

            apply_migrations_up_to(&second_migration_name)?;

            // Check db schema
            let table_definitions = get_surrealdb_table_definitions(None).await?;
            ensure!(
                table_definitions.len() == 7,
                "First run, second migration: wrong number of tables"
            );

            std::thread::sleep(std::time::Duration::from_secs(1));

            // Last migration
            add_archive_schema_file()?;
            add_archive_migration_file()?;

            let third_migration_name = get_third_migration_name()?;

            apply_migrations()?;

            // Check db schema
            let table_definitions = get_surrealdb_table_definitions(None).await?;
            ensure!(
                table_definitions.len() == 8,
                "First run, last migration: wrong number of tables"
            );

            // Check definition files
            const INITIAL_DEFINITION_FILE_PATH: &str =
                "tests-files/migrations/definitions/_initial.json";

            let second_migration_definition_file_path: String = format!(
                "tests-files/migrations/definitions/{}.json",
                &second_migration_name
            );

            let third_migration_definition_file_path: String = format!(
                "tests-files/migrations/definitions/{}.json",
                &third_migration_name
            );

            ensure!(Path::new(INITIAL_DEFINITION_FILE_PATH).exists());
            ensure!(Path::new(&second_migration_definition_file_path).exists());
            ensure!(Path::new(&third_migration_definition_file_path).exists());

            Ok(())
        })
    })
    .await?;

    run_with_surreal_instance_async(|| {
        Box::pin(async {
            // First migration
            let first_migration_name = get_first_migration_name()?;
            apply_migrations_up_to(&first_migration_name)?;

            // Check db schema
            let table_definitions = get_surrealdb_table_definitions(None).await?;
            ensure!(
                table_definitions.len() == 6,
                "Second run, first migration: wrong number of tables"
            );

            // Check data
            let is_table_empty = is_surreal_table_empty(None, "post").await?;
            ensure!(
                !is_table_empty,
                "First migration: 'post' table should not be empty"
            );

            let is_table_empty = is_surreal_table_empty(None, "category").await?;
            ensure!(
                is_table_empty,
                "First migration: 'category' table should be empty"
            );

            let is_table_empty = is_surreal_table_empty(None, "archive").await?;
            ensure!(
                is_table_empty,
                "First migration: 'archive' table should be empty"
            );

            // Second migration
            let second_migration_name = get_second_migration_name()?;
            apply_migrations_up_to(&second_migration_name)?;

            // Check db schema
            let table_definitions = get_surrealdb_table_definitions(None).await?;
            ensure!(
                table_definitions.len() == 7,
                "Second run, second migration: wrong number of tables"
            );

            // Check data
            let is_table_empty = is_surreal_table_empty(None, "post").await?;
            ensure!(
                !is_table_empty,
                "Second migration: 'post' table should not be empty"
            );

            let is_table_empty = is_surreal_table_empty(None, "category").await?;
            ensure!(
                !is_table_empty,
                "Second migration: 'category' table should not be empty"
            );

            let is_table_empty = is_surreal_table_empty(None, "archive").await?;
            ensure!(
                is_table_empty,
                "Second migration: 'archive' table should be empty"
            );

            // Last migration
            let third_migration_name = get_third_migration_name()?;
            apply_migrations()?;

            // Check db schema
            let table_definitions = get_surrealdb_table_definitions(None).await?;
            ensure!(
                table_definitions.len() == 8,
                "Second run, last migration: wrong number of tables"
            );

            // Check data
            let is_table_empty = is_surreal_table_empty(None, "post").await?;
            ensure!(
                !is_table_empty,
                "Last migration: 'post' table should not be empty"
            );

            let is_table_empty = is_surreal_table_empty(None, "category").await?;
            ensure!(
                !is_table_empty,
                "Last migration: 'category' table should not be empty"
            );

            let is_table_empty = is_surreal_table_empty(None, "archive").await?;
            ensure!(
                !is_table_empty,
                "Last migration: 'archive' table should not be empty"
            );

            // Check definition files
            const INITIAL_DEFINITION_FILE_PATH: &str =
                "tests-files/migrations/definitions/_initial.json";

            let second_migration_definition_file_path: String = format!(
                "tests-files/migrations/definitions/{}.json",
                &second_migration_name
            );

            let third_migration_definition_file_path: String = format!(
                "tests-files/migrations/definitions/{}.json",
                &third_migration_name
            );

            ensure!(Path::new(INITIAL_DEFINITION_FILE_PATH).exists());
            ensure!(Path::new(&second_migration_definition_file_path).exists());
            ensure!(Path::new(&third_migration_definition_file_path).exists());

            Ok(())
        })
    })
    .await
}

#[tokio::test]
#[serial]
async fn apply_3_consecutives_schema_and_data_changes_then_down_to_previous_migration() -> Result<()>
{
    clear_tests_files()?;
    scaffold_blog_template()?;
    empty_folder("tests-files/migrations")?;

    run_with_surreal_instance_async(|| {
        Box::pin(async {
            // First migration
            add_post_migration_file()?;
            let first_migration_name = get_first_migration_name()?;
            write_post_migration_down_file(&first_migration_name)?;

            apply_migrations_up_to(&first_migration_name)?;

            std::thread::sleep(std::time::Duration::from_secs(1));

            // Second migration
            add_category_schema_file()?;
            add_category_migration_file()?;
            let second_migration_name = get_second_migration_name()?;
            write_category_migration_down_file(&second_migration_name)?;

            apply_migrations_up_to(&second_migration_name)?;

            std::thread::sleep(std::time::Duration::from_secs(1));

            // Last migration
            add_archive_schema_file()?;
            add_archive_migration_file()?;
            let third_migration_name = get_third_migration_name()?;
            write_archive_migration_down_file(&third_migration_name)?;

            apply_migrations()?;

            // Down to last migration
            apply_migrations_down(&second_migration_name)?;

            // Check data
            let is_table_empty = is_surreal_table_empty(None, "post").await?;
            ensure!(!is_table_empty, "'post' table should not be empty");

            let is_table_empty = is_surreal_table_empty(None, "category").await?;
            ensure!(!is_table_empty, "'category' table should not be empty");

            let is_table_empty = is_surreal_table_empty(None, "archive").await?;
            ensure!(is_table_empty, "'archive' table should be empty");

            // Check db schema
            let table_definitions = get_surrealdb_table_definitions(None).await?;
            ensure!(table_definitions.len() == 7);

            Ok(())
        })
    })
    .await
}

const INITIAL_DEFINITION_SCHEMAS: &str = "# in: user
# out: post, comment
DEFINE TABLE comment SCHEMALESS;

DEFINE FIELD content ON comment TYPE string ASSERT $value != NONE;
DEFINE FIELD created_at ON comment TYPE datetime VALUE $before OR time::now();
DEFINE TABLE post SCHEMALESS;

DEFINE FIELD title ON post TYPE string;
DEFINE FIELD content ON post TYPE string;
DEFINE FIELD author ON post TYPE record (user) ASSERT $value != NONE;
DEFINE FIELD created_at ON post TYPE datetime VALUE $before OR time::now();
DEFINE FIELD status ON post TYPE string VALUE $value OR $before OR 'DRAFT' ASSERT $value == NONE OR $value INSIDE ['DRAFT', 'PUBLISHED'];
DEFINE TABLE script_migration SCHEMAFULL;

DEFINE FIELD script_name ON script_migration TYPE string;
DEFINE FIELD executed_at ON script_migration TYPE datetime VALUE $before OR time::now();
DEFINE TABLE user SCHEMALESS;

DEFINE FIELD username ON user TYPE string ASSERT $value != NONE;
DEFINE FIELD email ON user TYPE string ASSERT is::email($value);
DEFINE FIELD password ON user TYPE string ASSERT $value != NONE;
DEFINE FIELD registered_at ON user TYPE datetime VALUE $before OR time::now();";

const INITIAL_DEFINITION_EVENTS: &str = "DEFINE TABLE publish_post SCHEMALESS;

DEFINE FIELD post_id ON publish_post;
DEFINE FIELD created_at ON publish_post TYPE datetime VALUE $before OR time::now();

DEFINE EVENT publish_post ON TABLE publish_post WHEN $before == NONE THEN (
    UPDATE post SET status = \"PUBLISHED\" WHERE id = $after.post_id
);
DEFINE TABLE unpublish_post SCHEMALESS;

DEFINE FIELD post_id ON unpublish_post;
DEFINE FIELD created_at ON unpublish_post TYPE datetime VALUE $before OR time::now();

DEFINE EVENT unpublish_post ON TABLE unpublish_post WHEN $before == NONE THEN (
    UPDATE post SET status = \"DRAFT\" WHERE id = $after.post_id
);";

const SECOND_MIGRATION_SCHEMAS: &str = "--- original
+++ modified
@@ -1,3 +1,7 @@
+DEFINE TABLE category SCHEMALESS;
+
+DEFINE FIELD name ON category TYPE string;
+DEFINE FIELD created_at ON category TYPE datetime VALUE $before OR time::now();
 # in: user
 # out: post, comment
 DEFINE TABLE comment SCHEMALESS;\n";

const THIRD_MIGRATION_SCHEMAS: &str = "--- original
+++ modified
@@ -1,3 +1,9 @@
+DEFINE TABLE archive SCHEMALESS;
+
+DEFINE FIELD name ON archive TYPE string;
+DEFINE FIELD from_date ON archive TYPE datetime;
+DEFINE FIELD to_date ON archive TYPE datetime;
+DEFINE FIELD created_at ON archive TYPE datetime VALUE $before OR time::now();
 DEFINE TABLE category SCHEMALESS;

 DEFINE FIELD name ON category TYPE string;\n";
