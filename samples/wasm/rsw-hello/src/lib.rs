use std::path::Path;
use surrealdb::{
    engine::any::{connect, Any},
    Surreal,
};
use surrealdb_migrations::SurrealdbMigrations;
use utils::set_panic_hook;
use wasm_bindgen::prelude::*;

mod utils;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, rsw-hello!");
}

#[wasm_bindgen]
pub async fn setup() {
    set_panic_hook();

    let db = connect("indxdb://HelloDb")
        .await
        .expect("Failed to connect to database");

    db.use_ns("test")
        .use_db("test")
        .await
        .expect("Failed to use namespace");

    SurrealdbMigrations::new(db)
        .up()
        .await
        .expect("Failed to apply migrations");
}
