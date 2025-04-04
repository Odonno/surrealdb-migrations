use assert_fs::TempDir;
use color_eyre::eyre::{ensure, Error, Result};
use insta::{assert_snapshot, Settings};

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
    ensure!(
        definitions_files.count() == 1,
        "Wrong number of definitions files"
    );

    ensure!(
        definitions_dir.join("_initial.json").exists(),
        "Initial definition file should exist"
    );

    Ok(())
}

#[tokio::test]
async fn apply_3_consecutives_schema_and_data_changes() -> Result<()> {
    let insta_settings = Settings::new();

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
    ensure!(
        definitions_files.count() == 1,
        "Wrong number of definitions files"
    );

    let initial_definition_file_path = definitions_dir.join("_initial.json");

    ensure!(
        initial_definition_file_path.exists(),
        "Initial definition file should exist"
    );

    let initial_migration_definition_str = std::fs::read_to_string(&initial_definition_file_path)?;
    let initial_migration_definition =
        serde_json::from_str::<MigrationDefinition>(&initial_migration_definition_str)?;

    insta_settings.bind(|| {
        assert_snapshot!(
            "Initial migration schemas",
            initial_migration_definition.schemas.unwrap_or_default()
        );
        assert_snapshot!(
            "Initial migration events",
            initial_migration_definition.events.unwrap_or_default()
        );
        Ok::<(), Error>(())
    })?;

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
    ensure!(
        definitions_files.count() == 2,
        "Wrong number of definitions files"
    );

    let second_migration_definition_file_path =
        definitions_dir.join(format!("{second_migration_name}.json"));

    ensure!(
        initial_definition_file_path.exists(),
        "Initial definition file should exist"
    );
    ensure!(
        second_migration_definition_file_path.exists(),
        "Second definition file should exist"
    );

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

    insta_settings.bind(|| {
        assert_snapshot!(
            "Second migration",
            second_migration_definition.schemas.unwrap_or_default()
        );
        Ok::<(), Error>(())
    })?;
    ensure!(
        second_migration_definition.events.is_none(),
        "Second migration: wrong events"
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
    ensure!(
        definitions_files.count() == 3,
        "Wrong number of definitions files"
    );

    let third_migration_definition_file_path =
        definitions_dir.join(format!("{third_migration_name}.json"));

    ensure!(
        initial_definition_file_path.exists(),
        "Initial definition file should exist"
    );
    ensure!(
        second_migration_definition_file_path.exists(),
        "Second definition file should exist"
    );
    ensure!(
        third_migration_definition_file_path.exists(),
        "Third definition file should exist"
    );

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

    insta_settings.bind(|| {
        assert_snapshot!(
            "Third migration",
            third_migration_definition.schemas.unwrap_or_default()
        );
        Ok::<(), Error>(())
    })?;
    ensure!(
        third_migration_definition.events.is_none(),
        "Third migration: wrong events"
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

        ensure!(
            initial_definition_file_path.exists(),
            "Initial definition file should exist"
        );
        ensure!(
            second_migration_definition_file_path.exists(),
            "Second definition file should exist"
        );
        ensure!(
            third_migration_definition_file_path.exists(),
            "Third definition file should exist"
        );
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

    ensure!(
        initial_definition_file_path.exists(),
        "Initial definition file should exist"
    );
    ensure!(
        second_migration_definition_file_path.exists(),
        "Second definition file should exist"
    );
    ensure!(
        third_migration_definition_file_path.exists(),
        "Third definition file should exist"
    );

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
    ensure!(table_definitions.len() == 8, "Wrong number of tables");

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
    ensure!(table_definitions.len() == 7, "Wrong number of tables");

    Ok(())
}
