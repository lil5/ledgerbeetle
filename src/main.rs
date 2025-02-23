use std::sync::Arc;
mod models;
mod routes;
mod schema;

use axum::{
    routing::{get, post, put},
    Router,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenvy::dotenv;
use hledger_tb::http_err;
use tigerbeetle_unofficial as tb;
extern crate clap;

// this embeds the migrations into the application binary
// the migration path is relative to the `CARGO_MANIFEST_DIR`
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

#[tokio::main]
async fn main() {
    // setup dot env
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let tb_cluster_id = std::env::var("TB_CLIENT_ID")
        .expect("TB_CLIENT_ID must be set")
        .parse()
        .expect("TB_CLIENT_ID must be a number");
    let tb_address = std::env::var("TB_ADDRESS").expect("TB_ADDRESS must be set");

    // setup connection pool
    let manager =
        deadpool_diesel::postgres::Manager::new(database_url, deadpool_diesel::Runtime::Tokio1);
    let pool = deadpool_diesel::postgres::Pool::builder(manager)
        .build()
        .unwrap();

    // run the migrations on server startup
    {
        let conn = pool.get().await.unwrap();
        conn.interact(|conn| conn.run_pending_migrations(MIGRATIONS).map(|_| ()))
            .await
            .unwrap()
            .unwrap();
    }

    let tb = Arc::new(
        tb::Client::new(tb_cluster_id, tb_address).expect("Unable to connect to tigerbeetle"),
    );

    let app_state = routes::AppState { pool, tb };

    let app = Router::new()
        .route("/", get(routes::get_index))
        .route("/accountnames", get(routes::get_account_names))
        .route("/add", post(routes::post_add))
        .route("/add", put(routes::post_add))
        .route("/version", get(routes::get_version))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
