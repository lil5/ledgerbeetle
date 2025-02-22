use axum::{extract::State, response::Redirect, Json};
use deadpool_diesel::postgres::Pool;
use regex::Regex;
use serde::Deserialize;
use std::sync::Arc;
use std::sync::LazyLock;
use tigerbeetle_unofficial as tb;
use validator::Validate;
use validator::ValidationError;

static RE_DATE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d{4}-\d\d-\d\d$").unwrap());

use crate::models::find_or_create_account;
use crate::models::read_amount;
use crate::models::Account;
use crate::{http_err, models};

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool,
    pub tb: Arc<tb::Client>,
}

pub async fn get_account_names(
    State(state): State<AppState>,
) -> Result<Json<Vec<String>>, http_err::HttpErr> {
    let conn = state.pool.get().await.map_err(http_err::internal_error)?;

    let accounts = models::list_all_accounts(&conn).await?;

    Ok(Json(accounts))
}

pub async fn get_index() -> Redirect {
    Redirect::temporary("/journal")
}

pub async fn get_version() -> Json<String> {
    return Json(clap::crate_version!().to_string());
}

#[derive(Deserialize, Validate)]
pub struct PostAddRequest {
    #[validate(regex(path=*RE_DATE))]
    date: String,
    #[validate(length(min = 1))]
    description: String,
    #[validate(length(min = 2))]
    account: Vec<String>,
    #[validate(length(min = 2))]
    amount: Vec<String>,
}
pub async fn post_add(
    Json(payload): Json<PostAddRequest>,
    State(state): State<AppState>,
) -> Result<Json<Vec<String>>, http_err::HttpErr> {
    payload.validate().map_err(http_err::bad_error)?;

    if payload.account.len() != payload.amount.len() {
        return Err(http_err::bad_error(ValidationError::new(
            "account must be the same length as amount",
        )));
    }

    let conn = state.pool.get().await.map_err(http_err::internal_error)?;

    let mut payload_transfers: Vec<(Account, i64, String)> = Vec::new();
    for (i, account_name) in payload.account.iter().enumerate() {
        let found_amount = payload.account.get(i).unwrap();
        let found_amount = read_amount(&found_amount)?;

        let account = find_or_create_account(
            &conn,
            &(state.tb),
            account_name.to_string(),
            found_amount.1.clone(),
        )
        .await?;

        payload_transfers.push((account, found_amount.0, found_amount.1));
    }

    let arr: Vec<String> = Vec::new();

    Ok(Json(arr))
}

// utils
