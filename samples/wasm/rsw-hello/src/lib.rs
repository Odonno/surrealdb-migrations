use include_dir::{include_dir, Dir};
use serde::{Deserialize, Serialize};
use surrealdb::{engine::any::connect, sql::Thing};
use surrealdb_migrations::MigrationRunner;
use utils::set_panic_hook;
use wasm_bindgen::prelude::*;

mod utils;

#[wasm_bindgen]
extern "C" {}

#[wasm_bindgen]
pub async fn setup() {
    set_panic_hook();

    const DB_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/db");

    let db = connect("indxdb://HelloDb")
        .await
        .expect("Failed to connect to database");

    db.use_ns("test")
        .use_db("test")
        .await
        .expect("Failed to use namespace");

    MigrationRunner::new(&db)
        .load_files(&DB_DIR)
        .up()
        .await
        .expect("Failed to apply migrations");
}

#[wasm_bindgen]
pub async fn get_blog_posts() -> js_sys::Array {
    set_panic_hook();

    let db = connect("indxdb://HelloDb")
        .await
        .expect("Failed to connect to database");

    db.use_ns("test")
        .use_db("test")
        .await
        .expect("Failed to use namespace");

    let posts: Vec<BlogPost> = db.select("post").await.expect("Failed to get blog posts");

    posts
        .into_iter()
        .map(|post| serde_wasm_bindgen::to_value(&post).unwrap())
        .collect::<js_sys::Array>()
}

#[wasm_bindgen]
#[derive(Debug, Serialize, Deserialize)]
struct BlogPost {
    id: Thing,
    title: String,
    content: String,
    status: String,
}
