use anyhow::{ensure, Result};
use assert_fs::TempDir;

use crate::helpers::*;

#[test]
fn replay_migrations_on_clean_db() -> Result<()> {
    let temp_dir = TempDir::new()?;

    scaffold_blog_template(&temp_dir)?;

    {
        let db_name = generate_random_db_name()?;

        add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
        apply_migrations(&temp_dir, &db_name)?;
    }

    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;

    let mut cmd = create_cmd(&temp_dir)?;

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

    let migrations_dir = temp_dir.join("migrations");
    let definitions_dir = migrations_dir.join("definitions");

    let definitions_files =
        std::fs::read_dir(&definitions_dir)?.filter(|entry| match entry.as_ref() {
            Ok(entry) => entry.path().is_file(),
            Err(_) => false,
        });
    ensure!(definitions_files.count() == 1);

    ensure!(definitions_dir.join("_initial.json").exists());

    Ok(())
}

#[tokio::test]
async fn apply_3_consecutives_schema_and_data_changes() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir)?;
    empty_folder(&temp_dir.join("migrations"))?;

    // First migration
    add_post_migration_file(&temp_dir)?;

    let first_migration_name = get_first_migration_name(&temp_dir)?;

    apply_migrations_up_to(&temp_dir, &db_name, &first_migration_name)?;

    let migrations_dir = temp_dir.join("migrations");
    let definitions_dir = migrations_dir.join("definitions");

    // Check definitions files
    let definitions_files =
        std::fs::read_dir(&definitions_dir)?.filter(|entry| match entry.as_ref() {
            Ok(entry) => entry.path().is_file(),
            Err(_) => false,
        });
    ensure!(definitions_files.count() == 1);

    let initial_definition_file_path = definitions_dir.join("_initial.json");

    ensure!(initial_definition_file_path.exists());

    let initial_migration_definition_str = std::fs::read_to_string(&initial_definition_file_path)?;
    let initial_migration_definition =
        serde_json::from_str::<MigrationDefinition>(&initial_migration_definition_str)?;

    ensure!(initial_migration_definition.schemas == Some(INITIAL_DEFINITION_SCHEMAS.to_string()),);
    ensure!(initial_migration_definition.events == Some(INITIAL_DEFINITION_EVENTS.to_string()));

    // Check data
    let ns_db = Some(("test", db_name.as_str()));

    let is_table_empty = is_surreal_table_empty(ns_db, "post").await?;
    ensure!(
        !is_table_empty,
        "First migration: 'post' table should not be empty"
    );

    let is_table_empty = is_surreal_table_empty(ns_db, "category").await?;
    ensure!(
        is_table_empty,
        "First migration: 'category' table should be empty"
    );

    let is_table_empty = is_surreal_table_empty(ns_db, "archive").await?;
    ensure!(
        is_table_empty,
        "First migration: 'archive' table should be empty"
    );

    std::thread::sleep(std::time::Duration::from_secs(1));

    // Second migration
    add_category_schema_file(&temp_dir)?;
    add_category_migration_file(&temp_dir)?;

    let second_migration_name = get_second_migration_name(&temp_dir)?;

    apply_migrations_up_to(&temp_dir, &db_name, &second_migration_name)?;

    // Check definitions files
    let definitions_files =
        std::fs::read_dir(&definitions_dir)?.filter(|entry| match entry.as_ref() {
            Ok(entry) => entry.path().is_file(),
            Err(_) => false,
        });
    ensure!(definitions_files.count() == 2);

    let second_migration_definition_file_path =
        definitions_dir.join(format!("{second_migration_name}.json"));

    ensure!(initial_definition_file_path.exists());
    ensure!(second_migration_definition_file_path.exists());

    let new_initial_migration_definition_str =
        std::fs::read_to_string(&initial_definition_file_path)?;
    ensure!(
        initial_migration_definition_str == new_initial_migration_definition_str,
        "Second migration: Initial definition file should not have changed"
    );

    let second_migration_definition_str =
        std::fs::read_to_string(&second_migration_definition_file_path)?;
    let second_migration_definition =
        serde_json::from_str::<MigrationDefinition>(&second_migration_definition_str)?;

    ensure!(second_migration_definition.schemas == Some(SECOND_MIGRATION_SCHEMAS.to_string()));
    ensure!(second_migration_definition.events.is_none());

    // Check data
    let is_table_empty = is_surreal_table_empty(ns_db, "post").await?;
    ensure!(
        !is_table_empty,
        "Second migration: 'post' table should not be empty"
    );

    let is_table_empty = is_surreal_table_empty(ns_db, "category").await?;
    ensure!(
        !is_table_empty,
        "Second migration: 'category' table should not be empty"
    );

    let is_table_empty = is_surreal_table_empty(ns_db, "archive").await?;
    ensure!(
        is_table_empty,
        "Second migration: 'archive' table should be empty"
    );

    std::thread::sleep(std::time::Duration::from_secs(1));

    // Last migration
    add_archive_schema_file(&temp_dir)?;
    add_archive_migration_file(&temp_dir)?;

    let third_migration_name = get_third_migration_name(&temp_dir)?;

    apply_migrations(&temp_dir, &db_name)?;

    // Check definitions files
    let definitions_files =
        std::fs::read_dir(&definitions_dir)?.filter(|entry| match entry.as_ref() {
            Ok(entry) => entry.path().is_file(),
            Err(_) => false,
        });
    ensure!(definitions_files.count() == 3);

    let third_migration_definition_file_path =
        definitions_dir.join(format!("{third_migration_name}.json"));

    ensure!(initial_definition_file_path.exists());
    ensure!(second_migration_definition_file_path.exists());
    ensure!(third_migration_definition_file_path.exists());

    let new_initial_migration_definition_str =
        std::fs::read_to_string(initial_definition_file_path)?;
    ensure!(
        initial_migration_definition_str == new_initial_migration_definition_str,
        "Last migration: Initial definition file should not have changed"
    );

    let third_migration_definition_str =
        std::fs::read_to_string(&third_migration_definition_file_path)?;
    let third_migration_definition =
        serde_json::from_str::<MigrationDefinition>(&third_migration_definition_str)?;

    ensure!(third_migration_definition.schemas == Some(THIRD_MIGRATION_SCHEMAS.to_string()));
    ensure!(third_migration_definition.events.is_none());

    // Check data
    let is_table_empty = is_surreal_table_empty(ns_db, "post").await?;
    ensure!(
        !is_table_empty,
        "Last migration: 'post' table should not be empty"
    );

    let is_table_empty = is_surreal_table_empty(ns_db, "category").await?;
    ensure!(
        !is_table_empty,
        "Last migration: 'category' table should not be empty"
    );

    let is_table_empty = is_surreal_table_empty(ns_db, "archive").await?;
    ensure!(
        !is_table_empty,
        "Last migration: 'archive' table should not be empty"
    );

    Ok(())
}

#[tokio::test]
async fn apply_3_consecutives_schema_and_data_changes_on_clean_db() -> Result<()> {
    let temp_dir = TempDir::new()?;

    scaffold_blog_template(&temp_dir)?;
    empty_folder(&temp_dir.join("migrations"))?;

    {
        let db_name = generate_random_db_name()?;
        add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;

        let db_configuration = SurrealdbConfiguration {
            db: Some(db_name.to_string()),
            ..Default::default()
        };

        // First migration
        add_post_migration_file(&temp_dir)?;

        let first_migration_name = get_first_migration_name(&temp_dir)?;

        apply_migrations_up_to(&temp_dir, &db_name, &first_migration_name)?;

        // Check db schema
        let table_definitions =
            get_surrealdb_table_definitions(Some(db_configuration.clone())).await?;
        ensure!(
            table_definitions.len() == 7,
            "First run, first migration: wrong number of tables"
        );

        std::thread::sleep(std::time::Duration::from_secs(1));

        // Second migration
        add_category_schema_file(&temp_dir)?;
        add_category_migration_file(&temp_dir)?;

        let second_migration_name = get_second_migration_name(&temp_dir)?;

        apply_migrations_up_to(&temp_dir, &db_name, &second_migration_name)?;

        // Check db schema
        let table_definitions =
            get_surrealdb_table_definitions(Some(db_configuration.clone())).await?;
        ensure!(
            table_definitions.len() == 8,
            "First run, second migration: wrong number of tables"
        );

        std::thread::sleep(std::time::Duration::from_secs(1));

        // Last migration
        add_archive_schema_file(&temp_dir)?;
        add_archive_migration_file(&temp_dir)?;

        let third_migration_name = get_third_migration_name(&temp_dir)?;

        apply_migrations(&temp_dir, &db_name)?;

        // Check db schema
        let table_definitions =
            get_surrealdb_table_definitions(Some(db_configuration.clone())).await?;
        ensure!(
            table_definitions.len() == 9,
            "First run, last migration: wrong number of tables"
        );

        // Check definition files
        let migrations_dir = temp_dir.join("migrations");
        let definitions_dir = migrations_dir.join("definitions");

        let initial_definition_file_path = definitions_dir.join("_initial.json");

        let second_migration_definition_file_path =
            definitions_dir.join(format!("{second_migration_name}.json"));

        let third_migration_definition_file_path =
            definitions_dir.join(format!("{third_migration_name}.json"));

        ensure!(initial_definition_file_path.exists());
        ensure!(second_migration_definition_file_path.exists());
        ensure!(third_migration_definition_file_path.exists());
    }

    let db_name = generate_random_db_name()?;
    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;

    let db_configuration = SurrealdbConfiguration {
        db: Some(db_name.to_string()),
        ..Default::default()
    };

    let ns_db = Some(("test", db_name.as_str()));

    // First migration
    let first_migration_name = get_first_migration_name(&temp_dir)?;
    apply_migrations_up_to(&temp_dir, &db_name, &first_migration_name)?;

    // Check db schema
    let table_definitions = get_surrealdb_table_definitions(Some(db_configuration.clone())).await?;
    ensure!(
        table_definitions.len() == 7,
        "Second run, first migration: wrong number of tables"
    );

    // Check data
    let is_table_empty = is_surreal_table_empty(ns_db, "post").await?;
    ensure!(
        !is_table_empty,
        "First migration: 'post' table should not be empty"
    );

    let is_table_empty = is_surreal_table_empty(ns_db, "category").await?;
    ensure!(
        is_table_empty,
        "First migration: 'category' table should be empty"
    );

    let is_table_empty = is_surreal_table_empty(ns_db, "archive").await?;
    ensure!(
        is_table_empty,
        "First migration: 'archive' table should be empty"
    );

    // Second migration
    let second_migration_name = get_second_migration_name(&temp_dir)?;
    apply_migrations_up_to(&temp_dir, &db_name, &second_migration_name)?;

    // Check db schema
    let table_definitions = get_surrealdb_table_definitions(Some(db_configuration.clone())).await?;
    ensure!(
        table_definitions.len() == 8,
        "Second run, second migration: wrong number of tables"
    );

    // Check data
    let is_table_empty = is_surreal_table_empty(ns_db, "post").await?;
    ensure!(
        !is_table_empty,
        "Second migration: 'post' table should not be empty"
    );

    let is_table_empty = is_surreal_table_empty(ns_db, "category").await?;
    ensure!(
        !is_table_empty,
        "Second migration: 'category' table should not be empty"
    );

    let is_table_empty = is_surreal_table_empty(ns_db, "archive").await?;
    ensure!(
        is_table_empty,
        "Second migration: 'archive' table should be empty"
    );

    // Last migration
    let third_migration_name = get_third_migration_name(&temp_dir)?;
    apply_migrations(&temp_dir, &db_name)?;

    // Check db schema
    let table_definitions = get_surrealdb_table_definitions(Some(db_configuration.clone())).await?;
    ensure!(
        table_definitions.len() == 9,
        "Second run, last migration: wrong number of tables"
    );

    // Check data
    let is_table_empty = is_surreal_table_empty(ns_db, "post").await?;
    ensure!(
        !is_table_empty,
        "Last migration: 'post' table should not be empty"
    );

    let is_table_empty = is_surreal_table_empty(ns_db, "category").await?;
    ensure!(
        !is_table_empty,
        "Last migration: 'category' table should not be empty"
    );

    let is_table_empty = is_surreal_table_empty(ns_db, "archive").await?;
    ensure!(
        !is_table_empty,
        "Last migration: 'archive' table should not be empty"
    );

    // Check definition files
    let migrations_dir = temp_dir.join("migrations");
    let definitions_dir = migrations_dir.join("definitions");

    let initial_definition_file_path = definitions_dir.join("_initial.json");

    let second_migration_definition_file_path =
        definitions_dir.join(format!("{second_migration_name}.json"));

    let third_migration_definition_file_path =
        definitions_dir.join(format!("{third_migration_name}.json"));

    ensure!(initial_definition_file_path.exists());
    ensure!(second_migration_definition_file_path.exists());
    ensure!(third_migration_definition_file_path.exists());

    Ok(())
}

#[tokio::test]
async fn apply_3_consecutives_schema_and_data_changes_then_down_to_previous_migration() -> Result<()>
{
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir)?;
    empty_folder(&temp_dir.join("migrations"))?;

    // First migration
    add_post_migration_file(&temp_dir)?;
    let first_migration_name = get_first_migration_name(&temp_dir)?;
    write_post_migration_down_file(&temp_dir, &first_migration_name)?;

    apply_migrations_up_to(&temp_dir, &db_name, &first_migration_name)?;

    std::thread::sleep(std::time::Duration::from_secs(1));

    // Second migration
    add_category_schema_file(&temp_dir)?;
    add_category_migration_file(&temp_dir)?;
    let second_migration_name = get_second_migration_name(&temp_dir)?;
    write_category_migration_down_file(&temp_dir, &second_migration_name)?;

    apply_migrations_up_to(&temp_dir, &db_name, &second_migration_name)?;

    std::thread::sleep(std::time::Duration::from_secs(1));

    // Last migration
    add_archive_schema_file(&temp_dir)?;
    add_archive_migration_file(&temp_dir)?;
    let third_migration_name = get_third_migration_name(&temp_dir)?;
    write_archive_migration_down_file(&temp_dir, &third_migration_name)?;

    apply_migrations_up_to(&temp_dir, &db_name, &second_migration_name)?;

    // Down to last migration
    apply_migrations_down(&temp_dir, &db_name, &second_migration_name)?;

    // Check data
    let ns_db = Some(("test", db_name.as_str()));

    let is_table_empty = is_surreal_table_empty(ns_db, "post").await?;
    ensure!(!is_table_empty, "'post' table should not be empty");

    let is_table_empty = is_surreal_table_empty(ns_db, "category").await?;
    ensure!(!is_table_empty, "'category' table should not be empty");

    let is_table_empty = is_surreal_table_empty(ns_db, "archive").await?;
    ensure!(is_table_empty, "'archive' table should be empty");

    // Check db schema
    let db_configuration = SurrealdbConfiguration {
        db: Some(db_name.to_string()),
        ..Default::default()
    };

    let table_definitions = get_surrealdb_table_definitions(Some(db_configuration)).await?;
    ensure!(table_definitions.len() == 8);

    Ok(())
}

#[tokio::test]
async fn apply_3_consecutives_schema_and_data_changes_then_down_to_first_migration() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_name = generate_random_db_name()?;

    add_migration_config_file_with_db_name(&temp_dir, DbInstance::Root, &db_name)?;
    scaffold_blog_template(&temp_dir)?;
    empty_folder(&temp_dir.join("migrations"))?;

    // First migration
    add_post_migration_file(&temp_dir)?;
    let first_migration_name = get_first_migration_name(&temp_dir)?;
    write_post_migration_down_file(&temp_dir, &first_migration_name)?;

    apply_migrations_up_to(&temp_dir, &db_name, &first_migration_name)?;

    std::thread::sleep(std::time::Duration::from_secs(1));

    // Second migration
    add_category_schema_file(&temp_dir)?;
    add_category_migration_file(&temp_dir)?;
    let second_migration_name = get_second_migration_name(&temp_dir)?;
    write_category_migration_down_file(&temp_dir, &second_migration_name)?;

    apply_migrations_up_to(&temp_dir, &db_name, &second_migration_name)?;

    std::thread::sleep(std::time::Duration::from_secs(1));

    // Last migration
    add_archive_schema_file(&temp_dir)?;
    add_archive_migration_file(&temp_dir)?;
    let third_migration_name = get_third_migration_name(&temp_dir)?;
    write_archive_migration_down_file(&temp_dir, &third_migration_name)?;

    apply_migrations(&temp_dir, &db_name)?;

    // Down to first migration
    apply_migrations_down(&temp_dir, &db_name, "0")?;

    // Check data
    let ns_db = Some(("test", db_name.as_str()));

    let is_table_empty = is_surreal_table_empty(ns_db, "post").await?;
    ensure!(is_table_empty, "'post' table should be empty");

    let is_table_empty = is_surreal_table_empty(ns_db, "category").await?;
    ensure!(is_table_empty, "'category' table should be empty");

    let is_table_empty = is_surreal_table_empty(ns_db, "archive").await?;
    ensure!(is_table_empty, "'archive' table should be empty");

    // Check db schema
    let db_configuration = SurrealdbConfiguration {
        db: Some(db_name.to_string()),
        ..Default::default()
    };

    let table_definitions = get_surrealdb_table_definitions(Some(db_configuration)).await?;
    ensure!(table_definitions.len() == 7);

    Ok(())
}

const INITIAL_DEFINITION_SCHEMAS: &str = "# in: user
# out: post, comment
DEFINE TABLE comment SCHEMALESS
    PERMISSIONS
        FOR select FULL
        FOR create WHERE permission:create_comment IN $auth.permissions
        FOR update, delete WHERE in = $auth.id;

DEFINE FIELD content ON comment TYPE string ASSERT $value != NONE;
DEFINE FIELD created_at ON comment TYPE datetime VALUE $before OR time::now();
DEFINE TABLE permission SCHEMAFULL
    PERMISSIONS
        FOR select FULL
        FOR create, update, delete NONE;

DEFINE FIELD name ON permission TYPE string;
DEFINE FIELD created_at ON permission TYPE datetime VALUE $before OR time::now();

DEFINE INDEX unique_name ON permission COLUMNS name UNIQUE;
DEFINE TABLE post SCHEMALESS
    PERMISSIONS
        FOR select FULL
        FOR create WHERE permission:create_post IN $auth.permissions
        FOR update, delete WHERE author = $auth.id;

DEFINE FIELD title ON post TYPE string;
DEFINE FIELD content ON post TYPE string;
DEFINE FIELD author ON post TYPE record (user) ASSERT $value != NONE;
DEFINE FIELD created_at ON post TYPE datetime VALUE $before OR time::now();
DEFINE FIELD status ON post TYPE string VALUE $value OR $before OR 'DRAFT' ASSERT $value == NONE OR $value INSIDE ['DRAFT', 'PUBLISHED'];
DEFINE TABLE script_migration SCHEMAFULL
    PERMISSIONS
        FOR select FULL
        FOR create, update, delete NONE;

DEFINE FIELD script_name ON script_migration TYPE string;
DEFINE FIELD executed_at ON script_migration TYPE datetime VALUE $before OR time::now();
DEFINE TABLE user SCHEMAFULL
    PERMISSIONS
        FOR select FULL
        FOR update WHERE id = $auth.id
        FOR create, delete NONE;

DEFINE FIELD username ON user TYPE string ASSERT $value != NONE;
DEFINE FIELD email ON user TYPE string ASSERT is::email($value);
DEFINE FIELD password ON user TYPE string ASSERT $value != NONE;
DEFINE FIELD registered_at ON user TYPE datetime VALUE $before OR time::now();
DEFINE FIELD avatar ON user TYPE string;

DEFINE FIELD permissions ON user TYPE array VALUE [permission:create_post, permission:create_comment];
DEFINE FIELD permissions.* ON user TYPE record (permission);

DEFINE INDEX unique_username ON user COLUMNS username UNIQUE;
DEFINE INDEX unique_email ON user COLUMNS email UNIQUE;

DEFINE SCOPE user_scope
    SESSION 30d
    SIGNUP (
        CREATE user
        SET
            username = $username,
            email = $email,
            avatar = \"https://www.gravatar.com/avatar/\" + crypto::md5($email),
            password = crypto::argon2::generate($password)
    )
    SIGNIN (
        SELECT *
        FROM user
        WHERE username = $username AND crypto::argon2::compare(password, $password)
    );";

const INITIAL_DEFINITION_EVENTS: &str = "DEFINE TABLE publish_post SCHEMALESS
    PERMISSIONS
        FOR select, create FULL
        FOR update, delete NONE;

DEFINE FIELD post_id ON publish_post TYPE record(post);
DEFINE FIELD created_at ON publish_post TYPE datetime VALUE $before OR time::now();

DEFINE EVENT publish_post ON TABLE publish_post WHEN $before == NONE THEN (
    UPDATE post SET status = \"PUBLISHED\" WHERE id = $after.post_id
);
DEFINE TABLE unpublish_post SCHEMALESS
    PERMISSIONS
        FOR select, create FULL
        FOR update, delete NONE;

DEFINE FIELD post_id ON unpublish_post TYPE record(post);
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
 DEFINE TABLE comment SCHEMALESS\n";

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
