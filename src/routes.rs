use axum::extract::Path;
use axum::{extract::State, response::Redirect, Json};
use axum_macros::debug_handler;
use itertools::Itertools as _;
use serde::Deserialize;
use std::collections::HashMap;
use std::ops::Sub;
use tigerbeetle_unofficial as tb;
use validator::Validate;
use validator::ValidationError;
// static RE_DATE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d{4}-\d\d-\d\d$").unwrap());

use crate::http_err::HttpResult;
use crate::models::Account;
use crate::models::{find_accounts_re, TB_MAX_BATCH_SIZE};
use crate::models::{list_all_commodities, list_all_commodity_units};
use crate::responses::{RE_ACCOUNTS_FIND, RE_DATE};
use crate::tb_utils::u128::{from_hex_string, to_hex_string};
use crate::{http_err, models, responses, tb_utils, AppState};

pub async fn get_account_names(
    State(state): State<AppState>,
) -> http_err::HttpResult<Json<responses::ResponseAccountNames>> {
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
    Json(payload): Json<responses::RequestAdd>,
) -> http_err::HttpResult<Json<responses::ResponseAdd>> {
    if !state.allow_add {
        return Err(http_err::bad_error(std::io::Error::other(
            "writing to ledger is disabled",
        )));
    }

    payload.validate().map_err(http_err::bad_error)?;

    let conn = state.pool.get().await.map_err(http_err::internal_error)?;

    let mut tranfers: Vec<tb::Transfer> = Vec::new();
    let mut transfer_ids: Vec<String> = Vec::new();
    for (index, t) in payload.transactions.iter().enumerate() {
        let (account_debit, commodity) = models::find_or_create_account(
            Box::new(state.tb.clone()),
            &conn,
            t.debit_account.clone(),
            t.commodity_unit.clone(),
        )
        .await?;

        let (account_credit, _) = models::find_or_create_account(
            Box::new(state.tb.clone()),
            &conn,
            t.credit_account.clone(),
            t.commodity_unit.clone(),
        )
        .await?;

        let user_data_128 = tb_utils::u128::from_hex_string(&t.related_id);
        let user_data_64 = payload.full_date2.clone() as u64;

        let id = tb::id();
        transfer_ids.push(to_hex_string(id));

        let mut tranfer = tb::Transfer::new(id)
            .with_amount(t.amount as u128)
            .with_code(t.code as u16)
            .with_debit_account_id(from_hex_string(account_debit.tb_id.as_str()))
            .with_credit_account_id(from_hex_string(account_credit.tb_id.as_str()))
            .with_user_data_128(user_data_128)
            .with_user_data_64(user_data_64)
            .with_ledger(commodity.tb_ledger as u32);

        // forces all transfers to be a linked
        // see: https://docs.tigerbeetle.com/coding/linked-events/
        if payload.transactions.len() > 1 && index != payload.transactions.len() - 1 {
            tranfer = tranfer.with_flags(tb::transfer::Flags::LINKED)
        }
        tranfers.push(tranfer);
    }

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

    Ok(Json(transfer_ids))
}

#[derive(Deserialize, Validate)]
pub struct GetTransactionsRequest {
    #[validate(regex(path=*RE_DATE))]
    filter: String,
}
pub async fn get_transactions(
    Path(filter): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<responses::ResponseTransactions>, http_err::HttpErr> {
    let conn = state.pool.get().await.map_err(http_err::internal_error)?;

    if !RE_ACCOUNTS_FIND.is_match(&filter) {
        return Err(http_err::bad_error(ValidationError::new(
            "invalid accounts search",
        )));
    }

    let accounts: Vec<Account> = find_accounts_re(&conn, filter).await?;
    println!(
        "accounts found: {}",
        accounts.iter().map(|a| a.id).join(", ")
    );
    let commodities = list_all_commodities(&conn).await?;
    let commodities = commodities
        .iter()
        .map(|c| (c.tb_ledger as u32, c))
        .collect::<HashMap<_, _>>();

    println!(
        "commodities found: '{}'",
        commodities.iter().map(|a| a.0.clone()).join(", ")
    );

    // collect all transfers in a hashmap
    let mut transfers: HashMap<u128, tb::Transfer> = HashMap::new();
    for account in accounts.iter() {
        // get transfers per account
        let account_tb_id = from_hex_string(account.tb_id.as_str());

        let flags =
            tb::core::account::FilterFlags::DEBITS | tb::core::account::FilterFlags::CREDITS;
        let filter =
            tb::core::account::Filter::new(account_tb_id, TB_MAX_BATCH_SIZE).with_flags(flags);
        println!("getting account {account_tb_id} transfers");
        let transfers_data: Vec<tb::core::Transfer> = state
            .tb
            .read()
            .await
            .get_account_transfers(Box::new(filter))
            .await
            .map_err(http_err::internal_error)?;
        println!("found transfer data len {}", transfers_data.len());
        for transfer_data in transfers_data.iter() {
            let id = transfer_data.id();
            if !transfers.contains_key(&id) {
                transfers.insert(id, *transfer_data);
            }
        }
    }

    // collect all accounts
    let mut accounts = accounts
        .iter()
        .map(|a| (from_hex_string(a.tb_id.as_str()), a))
        .collect::<HashMap<u128, &Account>>();
    let mut missing_account_tb_ids: Vec<String> = Vec::new();
    for transfer in transfers
        .values()
        .sorted_by(|a, b| Ord::cmp(&a.timestamp(), &b.timestamp()))
    {
        for account_tb_id in vec![transfer.credit_account_id(), transfer.debit_account_id()].iter()
        {
            if !accounts.contains_key(account_tb_id) {
                missing_account_tb_ids.push(to_hex_string(*account_tb_id));
            }
        }
    }

    let more_accounts = models::find_accounts_by_tb_ids(&conn, missing_account_tb_ids).await?;
    more_accounts.iter().for_each(|a| {
        accounts.insert(from_hex_string(a.tb_id.as_str()), &a);
    });

    let transactions = transfers
        .iter()
        .sorted_by(|(_, a), (_, b)| Ord::cmp(&b.timestamp(), &a.timestamp()))
        .map(|(_, transfer)| {
            let commodity = commodities.get(&(transfer.ledger())).unwrap();
            responses::Transaction::from_tb(*transfer, Box::new(accounts.clone()), commodity)
                .map_err(|_| http_err::internal_error(ValidationError::new("err")))
        })
        .collect::<HttpResult<responses::ResponseTransactions>>()?;

    return Ok(Json(transactions));
}

pub async fn get_commodities(
    State(state): State<AppState>,
) -> Result<Json<responses::ResponseCommodities>, http_err::HttpErr> {
    let conn = state.pool.get().await.map_err(http_err::internal_error)?;
    let res = list_all_commodity_units(&conn).await?;

    Ok(Json(res))
}

pub async fn get_account_balances(
    Path(filter): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<responses::ResponseBalances>, http_err::HttpErr> {
    let conn = state.pool.get().await.map_err(http_err::internal_error)?;

    if !RE_ACCOUNTS_FIND.is_match(&filter) {
        return Err(http_err::bad_error(ValidationError::new(
            "invalid accounts search",
        )));
    }

    let accounts: Vec<Account> = find_accounts_re(&conn, filter).await?;
    println!(
        "accounts found: {}",
        accounts.iter().map(|a| a.id).join(", ")
    );
    let commodities = list_all_commodities(&conn).await?;
    let commodities = commodities
        .iter()
        .map(|c| (c.tb_ledger as u32, c))
        .collect::<HashMap<_, _>>();

    let mut balances: Vec<responses::Balance> = Vec::new();

    let ids = accounts
        .iter()
        .map(|a| from_hex_string(a.tb_id.as_str()))
        .collect::<Vec<_>>();

    let tb_accounts: Vec<tb::core::account::Account> = state
        .tb
        .read()
        .await
        .lookup_accounts(ids)
        .await
        .map_err(http_err::internal_error)?;

    for tb_account in tb_accounts.iter() {
        let amount = (tb_account.debits_posted() as i64).sub(tb_account.credits_posted() as i64);
        let tb_account_id = to_hex_string(tb_account.id());
        let account = accounts.iter().find(|a| a.tb_id == tb_account_id).unwrap();

        let commodity = commodities
            .iter()
            .find(|c| *(c.0) == (account.commodities_id as u32))
            .unwrap();
        let commodity_unit = commodity.1.unit.clone();
        let commodity_decimal = commodity.1.decimal_place.clone();

        balances.push(responses::Balance {
            account_name: account.name.clone(),
            amount,
            commodity_unit,
            commodity_decimal,
        });
    }

    Ok(Json(balances))
}
