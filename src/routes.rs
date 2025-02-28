use axum::extract::Path;
use axum::http::header::ACCEPT;
use axum::http::HeaderMap;
use axum::{extract::State, response::Redirect, Json};
use axum_macros::debug_handler;
use itertools::Itertools as _;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tigerbeetle_unofficial as tb;
use validator::Validate;
use validator::ValidationError;
// static RE_DATE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d{4}-\d\d-\d\d$").unwrap());

use crate::hledger::{Posting, RE_DATE};
use crate::http_err::HttpResult;
use crate::models::find_accounts_re;
use crate::models::Account;
use crate::models::{list_all_currencie_units, list_all_currencies};
use crate::tb_utils::u128::from_hex_string;
use crate::{hledger, http_err, models, tb_utils, AppState};

enum JsonOrString<T> {
    Json(Json<T>),
    String(String),
}

fn is_string_hledger<T>(headers: HeaderMap, t: T, f: fn(t: T) -> String) -> JsonOrString<T>
where
    T: Serialize,
{
    let ok = headers
        .get(ACCEPT)
        .and_then(|v| Some(v.to_str().unwrap().starts_with("text/hledger")))
        .unwrap_or(false);
    if ok {
        JsonOrString::String(f(t))
    } else {
        JsonOrString::Json(Json(t))
    }
}

pub async fn get_account_names(
    State(state): State<AppState>,
) -> http_err::HttpResult<Json<hledger::ResponseAccountNames>> {
    let conn = state.pool.get().await.map_err(http_err::internal_error)?;

    let accounts = models::list_all_accounts(&conn).await?;

    Ok(Json(accounts))
}

pub async fn get_index() -> Redirect {
    Redirect::temporary("/journal")
}

#[debug_handler]
pub async fn test() -> Json<Vec<Vec<i32>>> {
    let arr = vec![1, 2, 3, 4];
    let mut res: Vec<Vec<i32>> = Vec::new();
    for mut chunk in arr.into_iter().chunks(2).into_iter() {
        let r = chunk.by_ref().collect::<Vec<i32>>();

        res.push(r);
    }
    Json(res)
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

#[debug_handler]
pub async fn put_add(
    State(state): State<AppState>,
    Json(payload): Json<hledger::RequestAdd>,
) -> http_err::HttpResult<Json<()>> {
    if !state.allow_add {
        return Err(http_err::bad_error(std::io::Error::other(
            "writing to ledger is disabled",
        )));
    }
    let payload = vec![payload];
    // let payload = match payload {
    //     hledger::RequestAdd::RequestAddSingular(transaction) => vec![transaction],
    //     hledger::RequestAdd::RequestAddMultiple(transactions) => transactions,
    // };
    if payload.len() == 0 {
        return Err(http_err::bad_error(ValidationError::new(
            "Must contain at least one transaction",
        )));
    }

    payload.validate().map_err(http_err::bad_error)?;

    let conn = state.pool.get().await.map_err(http_err::internal_error)?;

    let mut tranfers: Vec<tb::Transfer> = Vec::new();
    for (index, transaction) in payload.iter().enumerate() {
        let tpostings = transaction.tpostings.clone();
        assert_eq!(tpostings.len() % 2, 0);
        let chunks: Vec<Vec<Posting>> = tpostings
            .into_iter()
            .chunks(2)
            .to_owned()
            .into_iter()
            .map(|chunk| chunk.collect::<Vec<Posting>>())
            .collect();
        for chunk in chunks.iter() {
            let posting_debit = chunk.get(1).unwrap();
            let posting_credit = chunk.get(0).unwrap();

            let first_pamount = posting_credit.pamount.iter().nth(0).unwrap();
            let (account_debit, _) = models::find_or_create_account(
                Box::new(state.tb.clone()),
                &conn,
                posting_debit.paccount.clone(),
                posting_debit
                    .pamount
                    .iter()
                    .nth(0)
                    .unwrap()
                    .acommodity
                    .clone(),
            )
            .await?;
            let (account_credit, currency) = models::find_or_create_account(
                Box::new(state.tb.clone()),
                &conn,
                posting_credit.paccount.clone(),
                first_pamount.acommodity.clone(),
            )
            .await?;

            let code = transaction.tcode.parse::<u16>().unwrap_or(1);

            let user_data_64 = transaction
                .tdate2
                .clone()
                .map(|d| d.parse::<u64>().unwrap_or(0))
                .unwrap_or(0);
            let user_data_128 = tb_utils::u128::from_hex_string(&transaction.tdescription);

            assert_ne!(account_credit.tb_id, account_debit.tb_id);

            let id = posting_credit.ptransaction.as_str();
            println!("transfer id: {id}");
            let mut tranfer = tb::Transfer::new(from_hex_string(id))
                .with_amount(
                    posting_debit
                        .pamount
                        .first()
                        .unwrap()
                        .aquantity
                        .decimal_mantissa as u128,
                )
                .with_code(code)
                .with_debit_account_id(account_debit.tb_id as u128)
                .with_credit_account_id(account_credit.tb_id as u128)
                .with_user_data_128(user_data_128)
                .with_user_data_64(user_data_64)
                .with_ledger(currency.tb_ledger as u32);

            // forces all transfers to be a linked
            // see: https://docs.tigerbeetle.com/coding/linked-events/
            if index == payload.len() - 1 && index != 0 {
                tranfer = tranfer.with_flags(tb::transfer::Flags::LINKED)
            }
            tranfers.push(tranfer);
        }
    }

    assert_eq!(
        tranfers.len() * 2,
        payload
            .iter()
            .map(|p| p.tpostings.len())
            .reduce(|acc, p| acc + p)
            .unwrap()
    );
    state
        .tb
        .read()
        .await
        .create_transfers(tranfers)
        .await
        .map_err(|e: tb::core::error::CreateTransfersError| {
            http_err::internal_error(format!(
                "error on adding transfers to tigerbeetle: {}",
                tb_utils::create_transfers_error_name(e)
            ))
        })?;

    Ok(Json(()))
}

#[derive(Deserialize, Validate)]
pub struct GetTransactionsRequest {
    #[validate(regex(path=*RE_DATE))]
    filter: String,
}
pub async fn get_transactions(
    Path(filter): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<hledger::ResponseTransactions>, http_err::HttpErr> {
    let conn = state.pool.get().await.map_err(http_err::internal_error)?;

    let accounts: Vec<Account> = find_accounts_re(&conn, filter).await?;
    let currencies = list_all_currencies(&conn).await?;
    let currencies = currencies
        .iter()
        .map(|c| {
            (
                c.tb_ledger as u32,
                (c, hledger::AmountStyle::from_tb(c.unit.clone())),
            )
        })
        .collect::<HashMap<_, _>>();

    // collect all transfers in a hashmap
    let mut transfers: HashMap<u128, tb::Transfer> = HashMap::new();
    for account in accounts.iter() {
        // get transfers per account
        let account_tb_id = account.tb_id.try_into().unwrap();

        let filter = tb::core::account::Filter::new(account_tb_id, u32::MAX);
        let transfers_data: Vec<tb::core::Transfer> = state
            .tb
            .read()
            .await
            .get_account_transfers(Box::new(filter))
            .await
            .map_err(http_err::internal_error)?;
        for transfer in transfers_data.iter() {
            let id = transfer.id();
            if !transfers.contains_key(&id) {
                transfers.insert(id, *transfer);
            }
        }
    }

    // collect all accounts
    let mut accounts = accounts
        .iter()
        .map(|a| (a.tb_id as u128, a))
        .collect::<HashMap<u128, &Account>>();
    let mut missing_account_tb_ids: Vec<i64> = Vec::new();
    for transfer in transfers.values() {
        for account_tb_id in vec![transfer.credit_account_id(), transfer.debit_account_id()].iter()
        {
            if !accounts.contains_key(account_tb_id) {
                missing_account_tb_ids.push(*account_tb_id as i64);
            }
        }
    }

    let more_accounts = models::find_accounts_by_tb_ids(&conn, missing_account_tb_ids).await?;
    more_accounts.iter().for_each(|a| {
        accounts.insert(a.tb_id as u128, &a);
    });

    let mut index = 0;
    let transactions = transfers
        .iter()
        .map(|(_, transfer)| {
            let (currency, amount_style) = currencies.get(&(transfer.ledger())).unwrap();
            hledger::Transaction::from_tb(
                *transfer,
                Box::new(accounts.clone()),
                currency,
                amount_style.clone(),
                &mut index,
            )
            .map_err(|_| http_err::internal_error(ValidationError::new("err")))
        })
        .collect::<HttpResult<hledger::ResponseTransactions>>()?;

    return Ok(Json(transactions));
}

pub async fn get_commodities(
    State(state): State<AppState>,
) -> Result<Json<hledger::ResponseCommodities>, http_err::HttpErr> {
    let conn = state.pool.get().await.map_err(http_err::internal_error)?;
    let res = list_all_currencie_units(&conn).await?;

    Ok(Json(res))
}
