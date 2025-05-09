//! An awesome SurrealDB migration tool, with a user-friendly CLI
//! and a versatile Rust library that enables seamless integration into any project.
//!
//! # The philosophy
//!
//! The SurrealDB Migrations project aims to simplify the creation of a SurrealDB database schema
//! and the evolution of the database through migrations.
//! A typical SurrealDB migration project is divided into 3 categories: schema, event and migration.
//!
//! A schema file represents no more than one SurrealDB table.
//! The list of schemas can be seen as the Query model (in a CQRS pattern).
//! The `schemas` folder can be seen as a view of the current data model.
//!
//! An event file represents no more than one SurrealDB event and the underlying table.
//! The list of events can be seen as the Command model (in a CQRS pattern).
//! The `events` folder can be seen as a view of the different ways to update the data model.
//!
//! A migration file represents a change in SurrealDB data.
//! It can be a change in the point of time between two schema changes.
//! Examples are:
//! when a column is renamed or dropped,
//! when a table is renamed or dropped,
//! when a new data is required (with default value), etc...
//!
//! # Get started
//!
//! ```rust,no_run
//! # use color_eyre::eyre::{eyre, ContextCompat, Result, WrapErr};
//! use surrealdb_migrations::MigrationRunner;
//! use surrealdb::engine::any::connect;
//! use surrealdb::opt::auth::Root;
//!
//! # #[tokio::main]
//! async fn main() -> Result<()> {
//!     let db = connect("ws://localhost:8000").await?;
//!
//!     // Signin as a namespace, database, or root user
//!     db.signin(Root {
//!         username: "root",
//!         password: "root",
//!     }).await?;
//!
//!     // Select a specific namespace / database
//!     db.use_ns("namespace").use_db("database").await?;
//!
//!     // Apply all migrations
//!     MigrationRunner::new(&db)
//!         .up()
//!         .await
//!         .expect("Failed to apply migrations");
//!
//!     Ok(())
//! }
//! ```

mod apply;
mod common;
mod config;
mod constants;
mod io;
mod models;
mod redo;
mod surrealdb;
mod tags;
mod validate_checksum;
mod validate_version_order;

use ::surrealdb::{Connection, Surreal};
use apply::ApplyArgs;
use color_eyre::eyre::Result;
use include_dir::Dir;
use models::{ApplyOperation, ScriptMigration};
use redo::RedoArgs;
use std::path::Path;
use validate_checksum::ValidateChecksumArgs;
use validate_version_order::ValidateVersionOrderArgs;

/// The main entry point for the library, used to apply migrations.
pub struct MigrationRunner<'a, C: Connection> {
    db: &'a Surreal<C>,
    dir: Option<&'a Dir<'static>>,
    config_file: Option<&'a Path>,
}

impl<'a, C: Connection> MigrationRunner<'a, C> {
    /// Create a new instance of `MigrationRunner`.
    ///
    /// ## Arguments
    ///
    /// * `db` - The SurrealDB instance used to apply migrations, etc...
    pub fn new(db: &'a Surreal<C>) -> Self {
        MigrationRunner {
            db,
            dir: None,
            config_file: None,
        }
    }

    /// Set path to the configuration file.
    /// By default, it will try to read configuration from the file `.surrealdb`.
    ///
    /// ## Arguments
    ///
    /// * `config_file` - Path to the configuration file.
    ///
    /// ## Examples
    ///
    /// ```rust,no_run
    /// # use color_eyre::eyre::{eyre, ContextCompat, Result, WrapErr};
    /// use include_dir::{include_dir, Dir};
    /// use surrealdb_migrations::MigrationRunner;
    /// use surrealdb::engine::any::connect;
    /// use surrealdb::opt::auth::Root;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// let db = connect("ws://localhost:8000").await?;
    ///
    /// // Signin as a namespace, database, or root user
    /// db.signin(Root {
    ///     username: "root",
    ///     password: "root",
    /// }).await?;
    ///
    /// // Select a specific namespace / database
    /// db.use_ns("namespace").use_db("database").await?;
    ///
    /// let runner = MigrationRunner::new(&db)
    ///     .use_config_file(&".surrealdb")
    ///     .up()
    ///     .await?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn use_config_file<P: AsRef<Path>>(self, config_file: &'a P) -> Self {
        MigrationRunner {
            db: self.db,
            dir: self.dir,
            config_file: Some(config_file.as_ref()),
        }
    }

    /// Load migration project files from an embedded directory.
    ///
    /// ## Arguments
    ///
    /// * `dir` - The directory containing the migration project files.
    ///
    /// ## Examples
    ///
    /// ```rust,no_run
    /// # use color_eyre::eyre::{eyre, ContextCompat, Result, WrapErr};
    /// use include_dir::{include_dir, Dir};
    /// use surrealdb_migrations::MigrationRunner;
    /// use surrealdb::engine::any::connect;
    /// use surrealdb::opt::auth::Root;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// let db = connect("ws://localhost:8000").await?;
    ///
    /// // Signin as a namespace, database, or root user
    /// db.signin(Root {
    ///     username: "root",
    ///     password: "root",
    /// }).await?;
    ///
    /// // Select a specific namespace / database
    /// db.use_ns("namespace").use_db("database").await?;
    ///
    /// // Load migration project files from an embedded directory
    /// const DB_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/blog");
    ///
    /// let runner = MigrationRunner::new(&db)
    ///     .load_files(&DB_DIR) // Will look for embedded files instead of the filesystem
    ///     .up()
    ///     .await?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn load_files(&'a self, dir: &'a Dir<'static>) -> Self {
        MigrationRunner {
            db: self.db,
            dir: Some(dir),
            config_file: self.config_file,
        }
    }

    /// Validate the version order of the migrations so that you cannot run migrations if there are
    /// gaps in the migrations history.
    ///
    /// ## Examples
    ///
    /// ```rust,no_run
    /// # use color_eyre::eyre::{eyre, ContextCompat, Result, WrapErr};
    /// use surrealdb_migrations::MigrationRunner;
    /// use surrealdb::engine::any::connect;
    /// use surrealdb::opt::auth::Root;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// let db = connect("ws://localhost:8000").await?;
    ///
    /// // Signin as a namespace, database, or root user
    /// db.signin(Root {
    ///     username: "root",
    ///     password: "root",
    /// }).await?;
    ///
    /// // Select a specific namespace / database
    /// db.use_ns("namespace").use_db("database").await?;
    ///
    /// let runner = MigrationRunner::new(&db);
    ///
    /// runner.validate_version_order().await?;
    /// runner.up().await?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub async fn validate_version_order(&self) -> Result<()> {
        let args = ValidateVersionOrderArgs {
            db: self.db,
            dir: self.dir,
            config_file: self.config_file,
        };
        validate_version_order::main(args).await
    }

    /// Validate the checksum of the migrations already played so that you cannot run migrations if there are
    /// changes detected in the migrations directory.
    ///
    /// ## Examples
    ///
    /// ```rust,no_run
    /// # use color_eyre::eyre::{eyre, ContextCompat, Result, WrapErr};
    /// use surrealdb_migrations::MigrationRunner;
    /// use surrealdb::engine::any::connect;
    /// use surrealdb::opt::auth::Root;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// let db = connect("ws://localhost:8000").await?;
    ///
    /// // Signin as a namespace, database, or root user
    /// db.signin(Root {
    ///     username: "root",
    ///     password: "root",
    /// }).await?;
    ///
    /// // Select a specific namespace / database
    /// db.use_ns("namespace").use_db("database").await?;
    ///
    /// let runner = MigrationRunner::new(&db);
    ///
    /// runner.validate_checksum().await?;
    /// runner.up().await?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub async fn validate_checksum(&self) -> Result<()> {
        let args = ValidateChecksumArgs {
            db: self.db,
            dir: self.dir,
            config_file: self.config_file,
        };
        validate_checksum::main(args).await
    }

    /// Apply schema definitions and apply all migrations.
    ///
    /// ## Examples
    ///
    /// ```rust,no_run
    /// # use color_eyre::eyre::{eyre, ContextCompat, Result, WrapErr};
    /// use surrealdb_migrations::MigrationRunner;
    /// use surrealdb::engine::any::connect;
    /// use surrealdb::opt::auth::Root;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// let db = connect("ws://localhost:8000").await?;
    ///
    /// // Signin as a namespace, database, or root user
    /// db.signin(Root {
    ///     username: "root",
    ///     password: "root",
    /// }).await?;
    ///
    /// // Select a specific namespace / database
    /// db.use_ns("namespace").use_db("database").await?;
    ///
    /// MigrationRunner::new(&db)
    ///     .up()
    ///     .await
    ///     .expect("Failed to apply migrations");
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub async fn up(&self) -> Result<()> {
        let args: ApplyArgs<C> = ApplyArgs {
            operation: ApplyOperation::Up,
            db: self.db,
            dir: self.dir,
            display_logs: false,
            dry_run: false,
            validate_checksum: false,
            validate_version_order: false,
            config_file: self.config_file,
            output: false,
        };
        apply::main(args).await
    }

    /// Apply schema definitions and all migrations up to and including the named migration.
    ///
    /// ## Arguments
    ///
    /// * `name` - This parameter allows you to skip ulterior migrations.
    ///
    /// ## Examples
    ///
    /// ```rust,no_run
    /// # use color_eyre::eyre::{eyre, ContextCompat, Result, WrapErr};
    /// use surrealdb_migrations::MigrationRunner;
    /// use surrealdb::engine::any::connect;
    /// use surrealdb::opt::auth::Root;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// let db = connect("ws://localhost:8000").await?;
    ///
    /// // Signin as a namespace, database, or root user
    /// db.signin(Root {
    ///     username: "root",
    ///     password: "root",
    /// }).await?;
    ///
    /// // Select a specific namespace / database
    /// db.use_ns("namespace").use_db("database").await?;
    ///
    /// MigrationRunner::new(&db)
    ///     .up_to("20230101_120002_AddPost")
    ///     .await
    ///     .expect("Failed to apply migrations");
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub async fn up_to(&self, name: &str) -> Result<()> {
        let args = ApplyArgs {
            operation: ApplyOperation::UpTo(name.to_string()),
            db: self.db,
            dir: self.dir,
            display_logs: false,
            dry_run: false,
            validate_checksum: false,
            validate_version_order: false,
            config_file: self.config_file,
            output: false,
        };
        apply::main(args).await
    }

    /// Apply only the next migration. This allows you to apply migrations step by step when necessary.
    ///
    /// ## Examples
    ///
    /// ```rust,no_run
    /// # use color_eyre::eyre::{eyre, ContextCompat, Result, WrapErr};
    /// use surrealdb_migrations::MigrationRunner;
    /// use surrealdb::engine::any::connect;
    /// use surrealdb::opt::auth::Root;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// let db = connect("ws://localhost:8000").await?;
    ///
    /// // Signin as a namespace, database, or root user
    /// db.signin(Root {
    ///     username: "root",
    ///     password: "root",
    /// }).await?;
    ///
    /// // Select a specific namespace / database
    /// db.use_ns("namespace").use_db("database").await?;
    ///
    /// MigrationRunner::new(&db)
    ///     .up_single()
    ///     .await
    ///     .expect("Failed to apply migration");
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub async fn up_single(&self) -> Result<()> {
        let args = ApplyArgs {
            operation: ApplyOperation::UpSingle,
            db: self.db,
            dir: self.dir,
            display_logs: false,
            dry_run: false,
            validate_checksum: false,
            validate_version_order: false,
            config_file: self.config_file,
            output: false,
        };
        apply::main(args).await
    }

    /// Revert schema definitions and all migrations down to the named migration.
    ///
    /// ## Arguments
    ///
    /// * `name` - This parameter allows you to revert applied migrations to this one.
    ///
    /// ## Examples
    ///
    /// ```rust,no_run
    /// # use color_eyre::eyre::{eyre, ContextCompat, Result, WrapErr};
    /// use surrealdb_migrations::MigrationRunner;
    /// use surrealdb::engine::any::connect;
    /// use surrealdb::opt::auth::Root;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// let db = connect("ws://localhost:8000").await?;
    ///
    /// // Signin as a namespace, database, or root user
    /// db.signin(Root {
    ///     username: "root",
    ///     password: "root",
    /// }).await?;
    ///
    /// // Select a specific namespace / database
    /// db.use_ns("namespace").use_db("database").await?;
    ///
    /// MigrationRunner::new(&db)
    ///     .down_to("20240607_163541_AddPost")
    ///     .await
    ///     .expect("Failed to revert migrations");
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub async fn down_to(&self, name: &str) -> Result<()> {
        let args = ApplyArgs {
            operation: ApplyOperation::DownTo(name.to_string()),
            db: self.db,
            dir: self.dir,
            display_logs: false,
            dry_run: false,
            validate_checksum: false,
            validate_version_order: false,
            config_file: self.config_file,
            output: false,
        };
        apply::main(args).await
    }

    /// Revert the last applied migration. This allows you to downgrade migrations step by step when necessary.
    ///
    /// ## Examples
    ///
    /// ```rust,no_run
    /// # use color_eyre::eyre::{eyre, ContextCompat, Result, WrapErr};
    /// use surrealdb_migrations::MigrationRunner;
    /// use surrealdb::engine::any::connect;
    /// use surrealdb::opt::auth::Root;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// let db = connect("ws://localhost:8000").await?;
    ///
    /// // Signin as a namespace, database, or root user
    /// db.signin(Root {
    ///     username: "root",
    ///     password: "root",
    /// }).await?;
    ///
    /// // Select a specific namespace / database
    /// db.use_ns("namespace").use_db("database").await?;
    ///
    /// MigrationRunner::new(&db)
    ///     .down_single()
    ///     .await
    ///     .expect("Failed to revert migration");
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub async fn down_single(&self) -> Result<()> {
        let args = ApplyArgs {
            operation: ApplyOperation::DownSingle,
            db: self.db,
            dir: self.dir,
            display_logs: false,
            dry_run: false,
            validate_checksum: false,
            validate_version_order: false,
            config_file: self.config_file,
            output: false,
        };
        apply::main(args).await
    }

    /// Revert ALL schema definitions and migrations.
    ///
    /// ## Examples
    ///
    /// ```rust,no_run
    /// # use color_eyre::eyre::{eyre, ContextCompat, Result, WrapErr};
    /// use surrealdb_migrations::MigrationRunner;
    /// use surrealdb::engine::any::connect;
    /// use surrealdb::opt::auth::Root;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// let db = connect("ws://localhost:8000").await?;
    ///
    /// // Signin as a namespace, database, or root user
    /// db.signin(Root {
    ///     username: "root",
    ///     password: "root",
    /// }).await?;
    ///
    /// // Select a specific namespace / database
    /// db.use_ns("namespace").use_db("database").await?;
    ///
    /// MigrationRunner::new(&db)
    ///     .reset()
    ///     .await
    ///     .expect("Failed to revert migrations");
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub async fn reset(&self) -> Result<()> {
        let args = ApplyArgs {
            operation: ApplyOperation::Reset,
            db: self.db,
            dir: self.dir,
            display_logs: false,
            dry_run: false,
            validate_checksum: false,
            validate_version_order: false,
            config_file: self.config_file,
            output: false,
        };
        apply::main(args).await
    }

    /// Re-apply an already applied migration script.
    ///
    /// ## Examples
    ///
    /// ```rust,no_run
    /// # use color_eyre::eyre::{eyre, ContextCompat, Result, WrapErr};
    /// use surrealdb_migrations::MigrationRunner;
    /// use surrealdb::engine::any::connect;
    /// use surrealdb::opt::auth::Root;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// let db = connect("ws://localhost:8000").await?;
    ///
    /// // Signin as a namespace, database, or root user
    /// db.signin(Root {
    ///     username: "root",
    ///     password: "root",
    /// }).await?;
    ///
    /// // Select a specific namespace / database
    /// db.use_ns("namespace").use_db("database").await?;
    ///
    /// MigrationRunner::new(&db)
    ///     .redo("20240607_163541_AddPost")
    ///     .await
    ///     .expect("Failed to re-apply migration");
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub async fn redo(&self, migration_script: &str) -> Result<()> {
        let args = RedoArgs {
            migration_script: migration_script.to_string(),
            db: self.db,
            dir: self.dir,
            display_logs: false,
            dry_run: false,
            validate_checksum: false,
            validate_version_order: false,
            config_file: self.config_file,
            output: false,
        };
        redo::main(args).await
    }

    /// List script migrations that have been applied to the database.
    ///
    /// ## Examples
    ///
    /// ```rust,no_run
    /// # use color_eyre::eyre::{eyre, ContextCompat, Result, WrapErr};
    /// use surrealdb_migrations::MigrationRunner;
    /// use surrealdb::engine::any::connect;
    /// use surrealdb::opt::auth::Root;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// let db = connect("ws://localhost:8000").await?;
    ///
    /// // Signin as a namespace, database, or root user
    /// db.signin(Root {
    ///     username: "root",
    ///     password: "root",
    /// }).await?;
    ///
    /// // Select a specific namespace / database
    /// db.use_ns("namespace").use_db("database").await?;
    ///
    /// let migrations_applied = MigrationRunner::new(&db)
    ///     .list()
    ///     .await?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list(&self) -> Result<Vec<ScriptMigration>> {
        surrealdb::list_script_migration_ordered_by_execution_date(self.db).await
    }
}
