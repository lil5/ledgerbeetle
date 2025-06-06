#![warn(clippy::unwrap_used)]

use std::sync::{Arc, LazyLock};
mod http_err;
mod models;
mod responses;

mod e2e_test;
mod routes;
mod schema;
mod tb_utils;

use axum::{
    routing::{get, post, put},
    Router,
};
use deadpool_diesel::postgres::Pool;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenvy::dotenv;
use regex::Regex;
use tigerbeetle_unofficial as tb;
use utoipa::OpenApi;

extern crate clap;

#[derive(OpenApi)]
#[openapi(paths(
    routes::mutate_migrate,
    routes::query_account_names_all,
    routes::query_export_hledger,
    routes::query_export_csv,
    routes::mutate_import_csv,
    routes::mutate_add,
    routes::query_prepare_add_fcfs,
    routes::query_account_transactions,
    routes::query_commodities_all,
    routes::query_account_balances,
    routes::query_account_income_statement,
    routes::get_openapi,
    routes::get_version,
))]
struct ApiDoc;

// this embeds the migrations into the application binary
// the migration path is relative to the `CARGO_MANIFEST_DIR`
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");
static RE_ENV_TRUE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(1|true|True|TRUE)$").expect("invalid regex"));

static RE_ENV_PORT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\d+$").expect("invalid regex"));

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool,
    pub tb: Arc<tb::Client>,
    pub allow_add: bool,
    pub allow_migrate: bool,
}

#[tokio::main]
async fn main() {
    // setup dot env
    dotenv().ok();

    let port = std::env::var("PORT")
        .and_then(|v| {
            if RE_ENV_PORT.is_match(v.as_str()) {
                panic!("port must be a number")
            }
            Ok(v)
        })
        .unwrap_or(String::from("8081"));

    let app = router().await;

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .expect("port must be available");
    axum::serve(listener, app)
        .await
        .expect("error on running axum serve");
}

pub async fn router() -> Router {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let tb_cluster_id = std::env::var("TB_CLIENT_ID")
        .expect("TB_CLIENT_ID must be set")
        .parse()
        .expect("TB_CLIENT_ID must be a number");
    let tb_address = std::env::var("TB_ADDRESS").expect("TB_ADDRESS must be set");
    let allow_add =
        RE_ENV_TRUE.is_match(&std::env::var("ALLOW_ADD").expect("ALLOW_ADD must be set"));
    let allow_migrate =
        RE_ENV_TRUE.is_match(&std::env::var("ALLOW_MIGRATE").expect("ALLOW_MIGRATE must be set"));
    if !allow_add && allow_migrate {
        panic!("ALLOW_ADD must be true if ALLOW_MIGRATE is true");
    }

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

    let tb = Arc::new(
        tb::Client::new(tb_cluster_id, tb_address).expect("Unable to connect to tigerbeetle"),
    );

    let app_state = AppState {
        pool,
        tb,
        allow_add,
        allow_migrate,
    };

    Router::new()
        .route("/mutate/migrate", put(routes::mutate_migrate))
        .route(
            "/query/account-names-all",
            post(routes::query_account_names_all),
        )
        .route("/mutate/add", put(routes::mutate_add))
        .route("/query/export-hledger", post(routes::query_export_hledger))
        .route("/query/export-csv", post(routes::query_export_csv))
        .route("/mutate/import-csv", put(routes::mutate_import_csv))
        .route("/query/prepare-add", post(routes::query_prepare_add_fcfs))
        .route(
            "/query/account-transactions",
            post(routes::query_account_transactions),
        )
        .route(
            "/query/commodities-all",
            post(routes::query_commodities_all),
        )
        .route(
            "/query/account-balances",
            post(routes::query_account_balances),
        )
        .route(
            "/query/account-income-statements",
            post(routes::query_account_income_statement),
        )
        .route("/openapi", get(routes::get_openapi))
        .route("/version", get(routes::get_version))
        .with_state(app_state)
}
