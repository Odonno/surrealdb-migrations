![Crates.io](https://img.shields.io/crates/v/surrealdb-migrations) ![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/Odonno/surrealdb-migrations/release.yml) ![GitHub](https://img.shields.io/github/license/Odonno/surrealdb-migrations) [![codecov](https://codecov.io/gh/Odonno/surrealdb-migrations/branch/main/graph/badge.svg?token=8DCQY63QP9)](https://codecov.io/gh/Odonno/surrealdb-migrations)

# SurrealDB Migrations

An awesome CLI for SurrealDB migrations (provides commands to scaffold, create and apply migrations).

```
cargo install surrealdb-migrations
```

> **Warning**
> This project is not production-ready, use at your own risk.

### The philosophy

The SurrealDB Migrations aims to simplify the creation of a SurrealDB database schema and the evolution of the database through migrations. A SurrealDB migration project is divided into 3 categories: schema, event and migration.

A schema file represents no more than one SurrealDB table. The list of schemas can be seen as the Query model (in a CQRS pattern). The `schemas` folder can be seen as a view of the current data model.

An event file represents no more than one SurrealDB event and the underlying table. The list of events can be seen as the Command model (in a CQRS pattern). The `events` folder can be seen as a view of the different ways to update the data model.

A migration file represents a change in SurrealDB data. It can be a change in the point of time between two schema changes. Examples are: when a column is renamed or dropped, when a table is renamed or dropped, when a new data is required (with default value), etc...

### Get started

You can start a migration project by scaffolding a new project using the following command line:

```
surrealdb-migrations scaffold empty
```

This will create the necessary folders and files in order to perform migrations. The `empty` template should look like this:

- /schemas
  - script_migration.surql
- /events
- /migrations

There are a number of pre-defined templates so you can play around and get started quickly.

Once you have created your migration project, you can start writing your own model. Based on the folders you saw earlier, you can create schema files, event files and migration files.

You can create strict schema files that represent tables stored in SurrealDB.

```
surrealdb-migrations create schema post --fields title,content,author,created_at,status
```

This will create a schemaless table with predefined fields:

```surql
DEFINE TABLE post SCHEMALESS;

DEFINE FIELD title ON post;
DEFINE FIELD content ON post;
DEFINE FIELD author ON post;
DEFINE FIELD created_at ON post;
DEFINE FIELD status ON post;
```

You can also create events in the same way.

```
surrealdb-migrations create event publish_post --fields post_id,created_at
```

This will define a table event with predefined fields:

```surql
DEFINE TABLE publish_post SCHEMALESS;

DEFINE FIELD post_id ON publish_post;
DEFINE FIELD created_at ON publish_post;

DEFINE EVENT publish_post ON TABLE publish_post WHEN $before == NONE THEN (
    # TODO
);
```

And when updating data, you can create migration files this way:

```
surrealdb-migrations create AddAdminUser
```

This will create a new file using the current date & time of the day, like `20230317_153201_AddAdminUser.surql` for example. All migrations files should be listed in a temporal order.

Finally, when you are ready, you can apply your schema and migrations to the database using the following command line:

```
surrealdb-migrations apply
```

### Predefined templates

To help you get started quickly, there is a list of predefined templates you can use:

- `empty` - The smallest migration project you can create, a clean schema with an already defined `script_migration` table to store the applied migrations
- `blog` - A blog domain model, with users having the ability to publish/unpublish posts and comments
- `ecommerce` - An ecommerce domain model, with customers having the ability to purchase products

You can scaffold a project using any of these templates using the following command line:

```
surrealdb-migrations scaffold <TEMPLATE>
```
