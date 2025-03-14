use anyhow::anyhow;
use axum::body::Body;
use axum::http::StatusCode;
use axum::response::Response;
use axum::{extract::State, Json};
// use axum_macros::debug_handler;
use itertools::Itertools as _;
use serde::Deserialize;
use std::collections::HashMap;
use std::ops::Sub;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tigerbeetle_unofficial as tb;
use utoipa::{OpenApi, ToSchema};
use validator::Validate;
use validator::ValidationError;

use crate::http_err::HttpResult;
use crate::models::Account;
use crate::models::{find_accounts_re, TB_MAX_BATCH_SIZE};
use crate::models::{list_all_commodities, list_all_commodity_units};
use crate::responses::RE_ACCOUNTS_GLOB;
use crate::tb_utils::u128::{from_hex_string, to_hex_string};
use crate::{http_err, models, responses, tb_utils, ApiDoc, AppState};

// #[debug_handler]
#[utoipa::path(put, path = "/mutate/migrate", responses(
    (status = 200, description = "Returns status 200 when migration is complete"),
    (status = 400, description = "Bad request error occurred", body = String),
    (status = 500, description = "Internal server error occurred", body = String),
))]
pub async fn mutate_migrate(
    State(state): State<AppState>,
    Json(body): Json<responses::RequestMigrate>,
) -> http_err::HttpResult<Json<()>> {
    if !state.allow_add {
        return Err(http_err::bad_error(std::io::Error::other(
            "writing to ledger is disabled",
        )));
    }

    body.validate().map_err(http_err::bad_error)?;

    let conn = state.pool.get().await.map_err(http_err::internal_error)?;
    let commodity_insert_res = models::insert_commodities(&conn, body.commodities).await?;

    let new_accounts: http_err::HttpResult<Vec<Account>> = body
        .accounts
        .iter()
        .map(|a| -> http_err::HttpResult<Account> {
            let find_commodity = commodity_insert_res
                .iter()
                .find(|c| c.id == a.c as i32)
                .ok_or(http_err::internal_error(anyhow!(
                    "commodity id listed in account that is not availible in commodities listing"
                )))?;

            Ok(Account {
                name: a.n.clone(),
                tb_id: a.t.clone(),
                commodities_id: find_commodity.id,
            })
        })
        .collect();
    let new_accounts = new_accounts?;

    models::insert_accounts(&conn, new_accounts).await?;

    Ok(Json(()))
}

#[utoipa::path(post, path = "/query/account-names-all", responses(
    (status = 200, description = "Returns list of transaction ids", body = responses::Vec<String>),
    (status = 400, description = "Bad request error occurred", body = String),
    (status = 500, description = "Internal server error occurred", body = String),
))]
pub async fn query_account_names_all(
    State(state): State<AppState>,
) -> http_err::HttpResult<Json<responses::ResponseAccountNames>> {
    let conn = state.pool.get().await.map_err(http_err::internal_error)?;

    let accounts = models::list_all_accounts(&conn).await?;

    Ok(Json(accounts))
}

// #[debug_handler]
#[utoipa::path(put, path = "/mutate/add", responses(
    (status = 200, description = "Returns list of transaction ids", body = responses::ResponseAdd),
    (status = 400, description = "Bad request error occurred", body = String),
    (status = 500, description = "Internal server error occurred", body = String),
))]
pub async fn mutate_add(
    State(state): State<AppState>,
    Json(body): Json<responses::RequestAdd>,
) -> http_err::HttpResult<Json<responses::ResponseAdd>> {
    if !state.allow_migrate {
        return Err(http_err::bad_error(std::io::Error::other(
            "migrating to ledger is disabled",
        )));
    }

    body.validate().map_err(http_err::bad_error)?;

    let conn = state.pool.get().await.map_err(http_err::internal_error)?;

    let mut tranfers: Vec<tb::Transfer> = Vec::new();
    let mut transfer_ids: Vec<String> = Vec::new();
    for (index, t) in body.transactions.iter().enumerate() {
        let (account_debit, commodity) = models::find_or_create_account(
            state.tb.clone(),
            &conn,
            t.debit_account.clone(),
            t.commodity_unit.clone(),
        )
        .await?;

        let (account_credit, _) = models::find_or_create_account(
            state.tb.clone(),
            &conn,
            t.credit_account.clone(),
            t.commodity_unit.clone(),
        )
        .await?;

        let user_data_128 = tb_utils::u128::from_hex_string(&t.related_id);
        let user_data_64 = body.full_date2 as u64;

        let id = tb::id();
        transfer_ids.push(to_hex_string(id));

        let mut tranfer = tb::Transfer::new(id)
            .with_amount(t.amount as u128)
            .with_code(t.code as u16)
            .with_debit_account_id(from_hex_string(account_debit.tb_id.as_str()))
            .with_credit_account_id(from_hex_string(account_credit.tb_id.as_str()))
            .with_user_data_128(user_data_128)
            .with_user_data_64(user_data_64)
            .with_ledger(commodity.id as u32);

        // forces all transfers to be a linked
        // see: https://docs.tigerbeetle.com/coding/linked-events/
        if body.transactions.len() > 1 && index != body.transactions.len() - 1 {
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

#[utoipa::path(post, path = "/query/prepare-add", responses(
    (status = 200, description = "Returns a prepared add payload to be run with the route PUT /app", body=responses::RequestAdd),
    (status = 400, description = "Bad request error occurred", body = String),
    (status = 500, description = "Internal server error occurred", body = String),
))]
pub async fn query_prepare_add_fcfs(
    State(state): State<AppState>,
    Json(body): Json<responses::RequestAddPrepareGlob>,
) -> http_err::HttpResult<Json<responses::ResponseAddPrepare>> {
    if !state.allow_add {
        return Err(http_err::bad_error(std::io::Error::other(
            "writing to ledger is disabled",
        )));
    }

    body.validate().map_err(http_err::bad_error)?;

    let conn = state.pool.get().await.map_err(http_err::internal_error)?;

    let mut add_transactions: Vec<responses::AddTransaction> = Vec::new();
    // map of key: account_tb_id value: balance
    let mut tb_account_balances: HashMap<String, i64> = HashMap::new();
    for t in body.filter_transactions.iter() {
        let mut remaining_amount = t.amount;
        'loop_credit_accounts_filter_item: for credit_accounts_filter_item in
            t.credit_accounts_filter.iter()
        {
            let credit_accounts = models::find_accounts_re_by_commodity(
                &conn,
                credit_accounts_filter_item.clone(),
                t.commodity_unit.clone(),
            )
            .await?;

            // println!("credit accounts {}", credit_accounts.len());

            // get account balances where not already retrieved
            let missing_tb_account_ids: Vec<u128> = credit_accounts
                .iter()
                .map(|a| &a.tb_id)
                .filter(|a| !tb_account_balances.contains_key(*a))
                .map(|s| tb_utils::u128::from_hex_string(s.as_str()))
                .collect();
            let tb_accounts: Vec<tb::core::account::Account> = state
                .tb
                .read()
                .await
                .lookup_accounts(missing_tb_account_ids)
                .await
                .map_err(http_err::internal_error)?;
            for a in tb_accounts.iter() {
                tb_account_balances.insert(
                    tb_utils::u128::to_hex_string(a.id()),
                    a.debits_posted() as i64 - a.credits_posted() as i64,
                );
            }

            // remove from account balance and until remaining amount is zero & add a transaction
            for account in credit_accounts.iter() {
                if let Some(tb_account_balance) = tb_account_balances.get_mut(&account.tb_id) {
                    if *tb_account_balance > 0 {
                        let add_transaction_amount = if remaining_amount < *tb_account_balance {
                            *tb_account_balance -= remaining_amount.clone();
                            let old_remaining_amount = remaining_amount.clone();
                            remaining_amount = 0;
                            old_remaining_amount
                        } else {
                            let old_tb_account_balance = tb_account_balance.clone();
                            remaining_amount -= old_tb_account_balance;
                            *tb_account_balance = 0;
                            old_tb_account_balance
                        };
                        add_transactions.push(responses::AddTransaction {
                            commodity_unit: t.commodity_unit.clone(),
                            code: t.code,
                            related_id: t.related_id.clone(),
                            debit_account: t.debit_account.clone(),
                            credit_account: account.name.clone(),
                            amount: add_transaction_amount,
                        });
                    }
                }
                if remaining_amount <= 0 {
                    break 'loop_credit_accounts_filter_item;
                }
            }
        }
        if remaining_amount > 0 {
            return Err(http_err::bad_error(anyhow!(
                "not enough inside credit accounts to build a transaction"
            )));
        }
    }

    fn assert_total_value(
        payload: Vec<responses::AddFilterTransaction>,
        add_transactions: Vec<responses::AddTransaction>,
    ) -> bool {
        let find_sum_amount_by_unit = add_transactions
            .iter()
            .chunk_by(|t| t.commodity_unit.clone())
            .into_iter()
            .map(|g| {
                (
                    g.0,
                    g.1.map(|t| t.amount).reduce(|acc, e| acc + e).unwrap_or(0),
                )
            })
            .collect::<HashMap<String, i64>>();
        for (unit, sum_amount) in find_sum_amount_by_unit.iter() {
            let payload_by_unit_sum_amount = payload
                .iter()
                .filter(|v| v.commodity_unit == *unit)
                .map(|v| v.amount)
                .reduce(|acc, v| acc + v)
                .unwrap_or(0);

            if *sum_amount != payload_by_unit_sum_amount {
                println!(
                    "for commodity {} payload sum amount {} is not the same as result amount {}",
                    unit, payload_by_unit_sum_amount, sum_amount
                );
                return false;
            }
        }
        true
    }
    assert!(assert_total_value(
        body.filter_transactions.clone(),
        add_transactions.clone()
    ));

    Ok(Json(responses::AddTransactions {
        full_date2: body.full_date2,
        transactions: add_transactions,
    }))
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct QueryTransactionsBody {
    date_newest: usize,
    date_oldest: usize,
    #[validate(regex(path=*RE_ACCOUNTS_GLOB))]
    accounts_glob: String,
}

#[utoipa::path(post, path = "/query/account-transactions", responses(
    (status = 200, description = "Returns list of transactions by filter", body=Vec<responses::Transaction>),
    (status = 400, description = "Bad request error occurred", body = String),
    (status = 500, description = "Internal server error occurred", body = String),
))]
pub async fn query_account_transactions(
    State(state): State<AppState>,
    Json(body): Json<QueryTransactionsBody>,
) -> Result<Json<responses::ResponseTransactions>, http_err::HttpErr> {
    body.validate().map_err(http_err::bad_error)?;

    let conn = state.pool.get().await.map_err(http_err::internal_error)?;

    let accounts: Vec<Account> = find_accounts_re(&conn, body.accounts_glob).await?;
    // println!(
    //     "accounts found: {}",
    //     accounts.iter().map(|a| a.id).join(", ")
    // );
    let commodities = list_all_commodities(&conn).await?;
    let commodities = commodities
        .iter()
        .map(|c| (c.id as u32, c))
        .collect::<HashMap<_, _>>();

    // println!(
    //     "commodities found: '{}'",
    //     commodities.iter().map(|a| a.0).join(", ")
    // );

    // collect all transfers in a hashmap
    let mut transfers: HashMap<u128, tb::Transfer> = HashMap::new();
    for account in accounts.iter() {
        // get transfers per account
        let account_tb_id = from_hex_string(account.tb_id.as_str());

        let flags = tb::core::account::FilterFlags::DEBITS
            | tb::core::account::FilterFlags::CREDITS
            | tb::core::account::FilterFlags::REVERSED;
        // println!("getting account {account_tb_id} transfers");

        // loops around and collects more than the TB_MAX_BATCH_SIZE if possible
        let mut is_response_larger_than_tb_max_batch_size = true;
        let mut previous_transfer_timestamp = UNIX_EPOCH
            .checked_add(Duration::from_millis(body.date_newest as u64))
            .expect("i64 unix nano date max");
        let oldest_transfer_timestamp = UNIX_EPOCH
            .checked_add(Duration::from_millis(body.date_oldest as u64))
            .expect("i64 unix nano date max");
        while is_response_larger_than_tb_max_batch_size {
            let filter = tb::core::account::Filter::new(account_tb_id, TB_MAX_BATCH_SIZE)
                .with_flags(flags)
                .with_timestamp_max(previous_transfer_timestamp)
                .with_timestamp_min(oldest_transfer_timestamp);
            let transfers_data: Vec<tb::core::Transfer> = state
                .tb
                .read()
                .await
                .get_account_transfers(Box::new(filter))
                .await
                .map_err(http_err::internal_error)?;
            // println!("found transfer data len {}", transfers_data.len());

            is_response_larger_than_tb_max_batch_size =
                transfers_data.len() > (TB_MAX_BATCH_SIZE as usize) - 1;
            let transfers_data_last_index = match transfers_data.len() {
                i if i > 0 => i - 1,
                _ => 0,
            };
            for (i, transfer_data) in transfers_data.iter().enumerate() {
                let id = transfer_data.id();
                transfers.entry(id).or_insert(*transfer_data);
                if transfers_data_last_index == i {
                    previous_transfer_timestamp = transfer_data
                        .timestamp()
                        .checked_sub(Duration::from_nanos(1))
                        .expect("time");
                }
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
        for account_tb_id in [transfer.credit_account_id(), transfer.debit_account_id()].iter() {
            if !accounts.contains_key(account_tb_id) {
                missing_account_tb_ids.push(to_hex_string(*account_tb_id));
            }
        }
    }

    let more_accounts = models::find_accounts_by_tb_ids(&conn, missing_account_tb_ids).await?;
    more_accounts.iter().for_each(|a| {
        accounts.insert(from_hex_string(a.tb_id.as_str()), a);
    });

    let transactions = transfers
        .iter()
        .sorted_by(|(_, a), (_, b)| Ord::cmp(&b.timestamp(), &a.timestamp()))
        .map(|(_, transfer)| {
            let commodity = commodities
                .get(&(transfer.ledger()))
                .expect("logical error unable to find commodity from transfer");
            responses::Transaction::from_tb(*transfer, accounts.clone(), commodity)
                .map_err(|_| http_err::internal_error(ValidationError::new("err")))
        })
        .collect::<HttpResult<responses::ResponseTransactions>>()?;

    // println!("transactions len {}", transactions.len());

    Ok(Json(transactions))
}

#[utoipa::path(post, path = "/query/commodities-all", responses(
    (status = 200, description = "Returns list of commodities", body=Vec<String>),
    (status = 400, description = "Bad request error occurred", body = String),
    (status = 500, description = "Internal server error occurred", body = String),
))]
pub async fn query_commodities_all(
    State(state): State<AppState>,
) -> Result<Json<responses::ResponseCommodities>, http_err::HttpErr> {
    let conn = state.pool.get().await.map_err(http_err::internal_error)?;
    let res = list_all_commodity_units(&conn).await?;

    Ok(Json(res))
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct QueryAccountBalancesBody {
    date: Option<usize>,
    #[validate(regex(path=*RE_ACCOUNTS_GLOB))]
    accounts_glob: String,
}

#[utoipa::path(post, path = "/query/account-balances", responses(
    (status = 200, description = "Returns list of account balances by filter", body=Vec<responses::Balance>),
    (status = 400, description = "Bad request error occurred", body = String),
    (status = 500, description = "Internal server error occurred", body = String),
))]
pub async fn query_account_balances(
    State(state): State<AppState>,
    Json(body): Json<QueryAccountBalancesBody>,
) -> Result<Json<responses::ResponseBalances>, http_err::HttpErr> {
    let conn = state.pool.get().await.map_err(http_err::internal_error)?;

    if !RE_ACCOUNTS_GLOB.is_match(&body.accounts_glob) {
        return Err(http_err::bad_error(ValidationError::new(
            "invalid accounts search",
        )));
    }

    let accounts: Vec<Account> = find_accounts_re(&conn, body.accounts_glob).await?;
    // println!(
    //     "accounts found: {}",
    //     accounts.iter().map(|a| a.id).join(", ")
    // );
    let commodities = list_all_commodities(&conn).await?;
    let commodities = commodities
        .iter()
        .map(|c| (c.id as u32, c))
        .collect::<HashMap<_, _>>();

    let mut balances: Vec<responses::Balance> = Vec::new();

    let ids = accounts
        .iter()
        .map(|a| from_hex_string(a.tb_id.as_str()))
        .collect::<Vec<_>>();

    if let Some(date) = body.date {
        // show balance by date
        let timestamp_max = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_millis(date as u64))
            .expect("invalid time")
            // fills nano seconds to max
            .checked_add(Duration::from_nanos(999_999))
            .expect("invalid time");
        for account in accounts.iter() {
            let filter = tb::account::Filter::new(
                tb_utils::u128::from_hex_string(account.tb_id.as_str()),
                1,
            )
            .with_flags(
                tb::account::FilterFlags::CREDITS
                    | tb::account::FilterFlags::DEBITS
                    | tb::account::FilterFlags::REVERSED,
            )
            .with_timestamp_max(timestamp_max);
            let tb_account_balance: Vec<tb::account::Balance> = state
                .tb
                .read()
                .await
                .get_account_balances(Box::new(filter))
                .await
                .map_err(http_err::internal_error)?;

            let commodity = commodities
                .iter()
                .find(|c| *(c.0) == (account.commodities_id as u32))
                .expect("logical error unable to find commodity");
            let commodity_unit = commodity.1.unit.clone();
            let commodity_decimal = commodity.1.decimal_place;

            let amount = match tb_account_balance.first() {
                Some(tb_account_balance_first) => (tb_account_balance_first.debits_posted() as i64)
                    .sub(tb_account_balance_first.credits_posted() as i64),
                None => 0,
            };

            balances.push(responses::Balance {
                account_name: account.name.clone(),
                amount,
                commodity_unit,
                commodity_decimal,
            });
        }
    } else {
        //show balance total
        let tb_accounts: Vec<tb::core::account::Account> = state
            .tb
            .read()
            .await
            .lookup_accounts(ids)
            .await
            .map_err(http_err::internal_error)?;

        for tb_account in tb_accounts.iter() {
            let amount =
                (tb_account.debits_posted() as i64).sub(tb_account.credits_posted() as i64);
            let tb_account_id = to_hex_string(tb_account.id());
            let account = accounts
                .iter()
                .find(|a| a.tb_id == tb_account_id)
                .expect("logical error unable to find account");

            let commodity = commodities
                .iter()
                .find(|c| *(c.0) == (account.commodities_id as u32))
                .expect("logical error unable to find commodity");
            let commodity_unit = commodity.1.unit.clone();
            let commodity_decimal = commodity.1.decimal_place;

            balances.push(responses::Balance {
                account_name: account.name.clone(),
                amount,
                commodity_unit,
                commodity_decimal,
            });
        }
    }

    Ok(Json(balances))
}

#[derive(Deserialize, Validate, ToSchema)]
pub struct QueryAccountIncomeStatementBody {
    #[validate(length(min = 1))]
    dates: Vec<usize>,
    #[validate(regex(path=*RE_ACCOUNTS_GLOB))]
    accounts_glob: String,
}

// #[debug_handler]
#[utoipa::path(post, path = "/query/account-income-statements", responses(
    (status = 200, description = "Returns list of balances by filter by date", body=responses::ResponseIncomeStatements),
    (status = 400, description = "Bad request error occurred", body = String),
    (status = 500, description = "Internal server error occurred", body = String),
))]
pub async fn query_account_income_statement(
    State(state): State<AppState>,
    Json(body): Json<QueryAccountIncomeStatementBody>,
) -> http_err::HttpResult<Json<responses::ResponseIncomeStatements>> {
    body.validate().map_err(http_err::bad_error)?;

    let conn = state.pool.get().await.map_err(http_err::internal_error)?;

    if !RE_ACCOUNTS_GLOB.is_match(&body.accounts_glob) {
        return Err(http_err::bad_error(ValidationError::new(
            "invalid accounts search",
        )));
    }

    let accounts: Vec<Account> = find_accounts_re(&conn, body.accounts_glob).await?;
    // println!(
    //     "accounts found: {}",
    //     accounts.iter().map(|a| a.id).join(", ")
    // );
    let commodities = list_all_commodities(&conn).await?;
    let commodities = commodities
        .iter()
        .map(|c| (c.id as u32, c))
        .collect::<HashMap<_, _>>();

    let dates: Vec<SystemTime> = body
        .dates
        .iter()
        .map(|date| {
            SystemTime::UNIX_EPOCH
                .checked_add(Duration::from_millis(*date as u64))
                .expect("invalid time")
                // fills nano seconds to max
                .checked_add(Duration::from_nanos(999_999))
                .expect("invalid time")
        })
        .collect();

    let mut income_statements: Vec<responses::IncomeStatement> = Vec::new();
    // show balance by date
    let dates_len = dates.len();
    for account in accounts.iter() {
        let mut amounts = Vec::with_capacity(dates_len);
        let commodity = commodities
            .iter()
            .find(|c| *(c.0) == (account.commodities_id as u32))
            .expect("logical error unable to find commodity");
        let commodity_unit = commodity.1.unit.clone();
        let commodity_decimal = commodity.1.decimal_place;
        for timestamp_max in dates.iter() {
            let filter = tb::account::Filter::new(
                tb_utils::u128::from_hex_string(account.tb_id.as_str()),
                1,
            )
            .with_flags(
                tb::account::FilterFlags::CREDITS
                    | tb::account::FilterFlags::DEBITS
                    | tb::account::FilterFlags::REVERSED,
            )
            .with_timestamp_max(*timestamp_max);
            let tb_account_balance: Vec<tb::account::Balance> = state
                .tb
                .read()
                .await
                .get_account_balances(Box::new(filter))
                .await
                .map_err(http_err::internal_error)?;

            let amount = match tb_account_balance.first() {
                Some(tb_account_balance_first) => (tb_account_balance_first.debits_posted() as i64)
                    .sub(tb_account_balance_first.credits_posted() as i64),
                None => 0,
            };
            amounts.push(amount);
        }
        income_statements.push(responses::IncomeStatement {
            account_name: account.name.clone(),
            amounts,
            commodity_unit,
            commodity_decimal,
        });
    }
    Ok(Json(responses::ResponseIncomeStatements {
        dates: body.dates,
        income_statements,
    }))
}

// #[debug_handler]
#[utoipa::path(get, path = "/openapi", responses(
    (status = 200, description = "Returns openapi v3.1 as json", body = String),
    (status = 500, description = "Internal server error occurred", body = String),
))]
pub async fn get_openapi() -> http_err::HttpResult<Response<Body>> {
    let openapi_json = ApiDoc::openapi()
        .to_pretty_json()
        .map_err(|e| http_err::bad_error(e))?;

    let res = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(openapi_json))
        .unwrap();
    Ok(res)
}

// #[debug_handler]
#[utoipa::path(get, path = "/version", responses(
    (status = 200, description = "Returns crate version", body = String),
))]
pub async fn get_version() -> Json<String> {
    Json(clap::crate_version!().to_string())
}
