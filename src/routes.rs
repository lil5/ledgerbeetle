use axum::extract::Path;
use axum::{extract::State, response::Redirect, Json};
use chrono::DateTime;
use deadpool_diesel::postgres::Pool;
use regex::Regex;
use serde::Deserialize;
use std::ops::Neg;
use std::sync::Arc;
use std::sync::LazyLock;
use tigerbeetle_unofficial as tb;
use validator::Validate;
use validator::ValidationError;

static RE_DATE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d{4}-\d\d-\d\d$").unwrap());

use crate::models::find_or_create_account;
use crate::models::list_all_currencies;
use crate::models::read_amount;
use crate::models::Account;
use crate::models::Currencies;
use crate::models::{create_transfer_details, find_accounts_re};
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
    State(state): State<AppState>,
    Json(payload): Json<PostAddRequest>,
) -> Result<String, http_err::HttpErr> {
    payload.validate().map_err(http_err::bad_error)?;

    if payload.account.len() != payload.amount.len() {
        return Err(http_err::bad_error(ValidationError::new(
            "account must be the same length as amount",
        )));
    }

    let conn = state.pool.get().await.map_err(http_err::internal_error)?;

    let mut payload_transfers: Vec<(Account, Currencies, i64, String)> = Vec::new();
    for (i, account_name) in payload.account.iter().enumerate() {
        let found_amount = payload.account.get(i).unwrap();
        let found_amount = read_amount(&found_amount)?;

        let (account, currency) = find_or_create_account(
            &conn,
            &(state.tb),
            account_name.to_string(),
            found_amount.1.clone(),
        )
        .await?;
        payload_transfers.push((account, currency, found_amount.0, found_amount.1));
    }

    let payee_per_unit: Vec<(i64, i32)> = payload_transfers
        .iter()
        .filter_map(|v| {
            if v.2 == 0 {
                Some((v.0.tb_id, v.1.tb_ledger))
            } else {
                None
            }
        })
        .collect();

    let user_data_64 = DateTime::parse_from_str(payload.date.as_str(), "%Y-%m-%d")
        .map_err(http_err::bad_error)?
        .timestamp()
        .try_into()
        .unwrap();
    let user_data_128 = tb::id();
    let mut tb_transfers: Vec<tb::Transfer> = Vec::new();

    let postings = payload_transfers.iter().filter(|v| v.2 != 0);

    // Chunk per tb_ledger then check that all amounts are all negative or positive
    for postings_by_unit in postings
        .clone()
        .collect::<Vec<_>>()
        .chunk_by(|a, b| a.1.tb_ledger == b.1.tb_ledger)
    {
        let mut is_negative: Option<bool> = None;
        for posting_is_negative in postings_by_unit.iter().map(|v| v.2.is_negative()) {
            match is_negative {
                None => is_negative = Some(posting_is_negative),
                Some(is_negative) => {
                    if posting_is_negative == is_negative {
                        return Err(http_err::bad_error(ValidationError::new(
                            "all amounts of each ledger must be all positive or negative",
                        )));
                    }
                }
            }
        }
    }

    for posting in postings {
        // find accound debit from payload with amount is zero and the tb_ledger is the same as the current item
        let payee = payee_per_unit
            .iter()
            .find(|d| d.1 == posting.1.tb_ledger)
            .ok_or(http_err::bad_error(ValidationError::new(
                "debit account not found in payload for this currency",
            )))?;

        let (account_debit, account_credit, amount) = {
            let is_amount_minus = posting.2.signum() == -1;
            let amount = if is_amount_minus {
                posting.2.neg()
            } else {
                posting.2
            };
            if is_amount_minus {
                (payee.0, posting.0.tb_id, amount)
            } else {
                (posting.0.tb_id, payee.0, amount)
            }
        };

        let tb_transfer = tb::Transfer::new(tb::id())
            .with_amount(amount.try_into().unwrap())
            .with_code(1)
            .with_debit_account_id(account_debit.try_into().unwrap())
            .with_credit_account_id(account_credit.try_into().unwrap())
            .with_user_data_64(user_data_64)
            .with_user_data_128(user_data_128)
            .with_ledger(payee.1.try_into().unwrap());

        tb_transfers.push(tb_transfer);
    }

    state
        .tb
        .create_transfers(tb_transfers)
        .await
        .map_err(http_err::internal_error)?;

    create_transfer_details(
        &conn,
        models::NewTransferDetail {
            tb_id: user_data_128.to_string(),
            description: payload.description,
        },
    )
    .await?;

    Ok("hi".to_string())
}

#[derive(Deserialize, Validate)]
pub struct GetTransactionsRequest {
    #[validate(regex(path=*RE_DATE))]
    filter: String,
}
pub async fn get_transactions(
    Path(filter): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<()>, http_err::HttpErr> {
    let conn = state.pool.get().await.map_err(http_err::internal_error)?;

    let accounts = find_accounts_re(&conn, filter).await?;

    for account in accounts.iter() {
        let account_tb_id = account.tb_id.try_into().unwrap();
        let filter = tb::core::account::Filter::new(account_tb_id, 1);
        state
            .tb
            .get_account_balances(Box::new(filter))
            .await
            .map_err(http_err::internal_error)?;
    }

    return Ok(Json(()));
}

pub async fn get_commodities(
    State(state): State<AppState>,
) -> Result<Json<Vec<String>>, http_err::HttpErr> {
    let conn = state.pool.get().await.map_err(http_err::internal_error)?;
    let res = list_all_currencies(&conn).await?;

    Ok(Json(res))
}
