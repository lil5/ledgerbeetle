#![warn(clippy::unwrap_used)]

use std::sync::{Arc, LazyLock};
mod http_err;
mod models;
mod responses;

mod routes;
mod schema;
mod tb_utils;

use axum::{
    routing::{get, put},
    Router,
};
use deadpool_diesel::postgres::Pool;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenvy::dotenv;
use regex::Regex;
use tigerbeetle_unofficial as tb;
use tokio::sync::RwLock;
extern crate clap;

// this embeds the migrations into the application binary
// the migration path is relative to the `CARGO_MANIFEST_DIR`
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");
static RE_ENV_TRUE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(1|true|True|TRUE)$").expect("invalid regex"));

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool,
    pub tb: Arc<RwLock<tb::Client>>,
    pub allow_add: bool,
}

#[tokio::main]
async fn main() {
    // setup dot env
    dotenv().ok();

    let app = router().await;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("port 3000 must be available");
    axum::serve(listener, app)
        .await
        .expect("error on running axum serve");
}

async fn router() -> Router {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let tb_cluster_id = std::env::var("TB_CLIENT_ID")
        .expect("TB_CLIENT_ID must be set")
        .parse()
        .expect("TB_CLIENT_ID must be a number");
    let tb_address = std::env::var("TB_ADDRESS").expect("TB_ADDRESS must be set");
    let allow_add =
        RE_ENV_TRUE.is_match(&std::env::var("ALLOW_ADD").expect("ALLOW_ADD must be set"));

    // setup connection pool
    let manager =
        deadpool_diesel::postgres::Manager::new(database_url, deadpool_diesel::Runtime::Tokio1);
    let pool = deadpool_diesel::postgres::Pool::builder(manager)
        .build()
        .expect("unable to connect to postgres");

    // run the migrations on server startup
    {
        let conn = pool
            .get()
            .await
            .expect("unable to connect to postgres pool");
        conn.interact(|conn| conn.run_pending_migrations(MIGRATIONS).map(|_| ()))
            .await
            .expect("unable to send request to postgres pool")
            .expect("error running migrations");
    }

    let tb = Arc::new(RwLock::new(
        tb::Client::new(tb_cluster_id, tb_address).expect("Unable to connect to tigerbeetle"),
    ));

    let app_state = AppState {
        pool,
        tb,
        allow_add,
    };

    Router::new()
        .route("/", get(routes::get_index))
        .route("/accountnames", get(routes::get_account_names))
        .route("/add", put(routes::put_add))
        .route("/test", get(routes::test))
        .route(
            "/accounttransactions/{filter}",
            get(routes::get_transactions),
        )
        .route(
            "/accountbalances/{filter}",
            get(routes::get_account_balances),
        )
        .route("/commodities", get(routes::get_commodities))
        .route("/version", get(routes::get_version))
        .with_state(app_state)
}

#[tokio::test]
async fn e2e() {
    use axum::http::StatusCode;
    use axum_test::TestServer;
    // setup dot env
    dotenv().ok();

    let app = router().await;

    let server = TestServer::new(app).unwrap();

    {
        let response = server.get("/").await;
        response.assert_status(StatusCode::TEMPORARY_REDIRECT);
    }

    {
        let response = server.get("/accountnames").await;
        let json = response.json::<responses::ResponseAccountNames>();
        assert!(json.iter().any(|v| v.starts_with("assets:")));
    }

    {
        let response = server.get("/commodities").await;
        let json = response.json::<responses::ResponseCommodities>();
        assert!(json.iter().any(|v| v == ""));
    }
}
