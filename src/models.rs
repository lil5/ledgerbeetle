use crate::{hledger, http_err};
use axum_macros::debug_handler;
use deadpool_diesel::postgres::Object;
use diesel::{dsl::Like, insert_into, prelude::*, result::Error::NotFound};
use itertools::Itertools;
use regex::Regex;
use std::sync::{Arc, LazyLock};
use tigerbeetle_unofficial as tb;
use tokio::sync::{RwLock, RwLockReadGuard};
use validator::ValidationError;

pub static TB_MAX_BATCH_SIZE: u32 = 8190;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::accounts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Account {
    pub id: i64,
    pub name: String,
    pub tb_id: i64,
}

pub async fn list_all_accounts(conn: &Object) -> Result<Vec<String>, http_err::HttpErr> {
    use crate::schema::accounts::dsl::*;
    conn.interact(|conn| {
        return accounts
            .select(name)
            .load::<String>(conn)
            .map_err(http_err::internal_error);
    })
    .await
    .map_err(http_err::internal_error)?
}

pub async fn find_or_create_account<'a>(
    tb: Box<Arc<RwLock<tb::Client>>>,
    conn: &Object,
    account_name: String,
    unit: String,
) -> http_err::HttpResult<(Account, Currencies)> {
    use crate::schema::accounts::dsl::*;

    let account_name_clone = account_name.clone();
    let first_account = conn
        .interact(|conn| {
            return accounts
                .select(Account::as_select())
                .filter(name.eq(account_name))
                .first(conn);
        })
        .await
        .map_err(http_err::internal_error)?;

    match first_account {
        Ok(v) => {
            let currency = find_or_create_currency(conn, unit.clone()).await?;
            Ok((v, currency))
        }
        Err(err) => {
            if err != NotFound {
                Err(http_err::internal_error(err))
            } else {
                println!("account {} not found creating...", account_name_clone);
                create_account(&conn, tb, account_name_clone, unit.clone()).await
            }
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name =  crate::schema::accounts)]
pub struct NewAccount<'a> {
    pub name: &'a str,
    pub currencies_id: &'a i32,
}
async fn create_account<'a>(
    conn: &Object,
    tb: Box<Arc<RwLock<tb::Client>>>,
    account_name: String,
    unit: String,
) -> Result<(Account, Currencies), http_err::HttpErr> {
    // return Err(http_err::internal_error(ValidationError::new("stuff")));
    let currency = find_or_create_currency(&conn, unit).await?;
    let account_name_clone = account_name.clone();
    let account_type = AccountType::read(account_name.as_str()).map_err(http_err::bad_error)?;
    println!("creating account_name: {}", account_name);
    let account = conn
        .interact(move |conn| {
            let new_account = NewAccount {
                name: account_name_clone.as_str(),
                currencies_id: &currency.id,
            };
            diesel::insert_into(crate::schema::accounts::table)
                .values(&new_account)
                .returning(Account::as_returning())
                .get_result(conn)
                .map_err(http_err::internal_error)
        })
        .await
        .map_err(http_err::internal_error)??;

    let flags = {
        let mut flags = tb::account::Flags::empty();
        let (disallow_red, disallow_green) = account_type.must_not_exceed();
        if disallow_green {
            flags &= tb::account::Flags::DEBITS_MUST_NOT_EXCEED_CREDITS
        }
        if disallow_red {
            flags &= tb::account::Flags::CREDITS_MUST_NOT_EXCEED_DEBITS
        }
        flags
    };

    let new_tb_account = tb::Account::new(
        account.tb_id.try_into().unwrap(),
        currency.tb_ledger.try_into().unwrap(),
        1,
    )
    .with_flags(flags);

    tb.read()
        .await
        .create_accounts(vec![new_tb_account])
        .await
        .map_err(http_err::internal_error)?;

    Ok((account, currency))
}

pub async fn find_accounts_re(
    conn: &Object,
    filter: String,
) -> Result<Vec<Account>, http_err::HttpErr> {
    use crate::schema::accounts::dsl::*;

    conn.interact(move |conn| {
        let filter = filter.replace("**", "%").replace("*", "_");
        let mut q = accounts.into_boxed();
        for (i, f) in filter.split("|").enumerate() {
            if i == 0 {
                q = q.filter(name.like(f));
            } else {
                q = q.or_filter(name.like(f));
            }
        }
        let q = q.select(Account::as_select());
        q.get_results::<Account>(conn)
            .map_err(http_err::internal_error)
    })
    .await
    .map_err(http_err::internal_error)?
}

pub async fn find_accounts_by_tb_ids(
    conn: &Object,
    tb_ids: Vec<i64>,
) -> http_err::HttpResult<Vec<Account>> {
    use crate::schema::accounts::dsl;

    conn.interact(|conn| {
        dsl::accounts
            .select(Account::as_select())
            .filter(dsl::tb_id.eq_any(tb_ids))
            .get_results::<Account>(conn)
            .map_err(http_err::internal_error)
    })
    .await
    .map_err(http_err::internal_error)?
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::currencies)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Currencies {
    pub id: i32,
    pub tb_ledger: i32,
    pub unit: String,
}

async fn find_or_create_currency(
    conn: &Object,
    unit: String,
) -> Result<Currencies, http_err::HttpErr> {
    use crate::schema::currencies::dsl;
    let unit_clone = unit.clone();
    let first_currency = conn
        .interact(|conn| {
            dsl::currencies
                .select(Currencies::as_select())
                .filter(dsl::unit.eq(unit))
                .first(conn)
        })
        .await
        .map_err(http_err::teapot_error)?;

    match first_currency {
        Ok(v) => Ok(v),
        Err(err) => {
            Err(http_err::teapot_error(err))
            // if err != NotFound {
            // } else {
            //     println!("{}", err);
            //     create_currency(&conn, unit_clone).await
            // }
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name =  crate::schema::currencies)]
pub struct NewCurrency<'a> {
    pub unit: &'a str,
}
async fn create_currency(conn: &Object, unit: String) -> Result<Currencies, http_err::HttpErr> {
    conn.interact(move |conn| {
        let new_currency = NewCurrency {
            unit: unit.as_str(),
        };
        return diesel::insert_into(crate::schema::currencies::table)
            .values(&new_currency)
            .returning(Currencies::as_returning())
            .get_result(conn)
            .map_err(http_err::internal_error);
    })
    .await
    .map_err(http_err::internal_error)?
}

pub async fn list_all_currencie_units(conn: &Object) -> Result<Vec<String>, http_err::HttpErr> {
    conn.interact(move |conn| {
        use crate::schema::currencies::dsl;
        return dsl::currencies
            .select(dsl::unit)
            .load(conn)
            .map_err(http_err::internal_error);
    })
    .await
    .map_err(http_err::internal_error)?
}
pub async fn list_all_currencies(conn: &Object) -> Result<Vec<Currencies>, http_err::HttpErr> {
    conn.interact(move |conn| {
        use crate::schema::currencies::dsl;
        return dsl::currencies
            .select(dsl::currencies::all_columns())
            .load(conn)
            .map_err(http_err::internal_error);
    })
    .await
    .map_err(http_err::internal_error)?
}

const ACCOUNT_TYPE_ASSETS: &str = "assets";
const ACCOUNT_TYPE_LIABILITIES: &str = "liabilities";
const ACCOUNT_TYPE_EQUITY: &str = "equity";
const ACCOUNT_TYPE_REVENUES: &str = "revenues";
const ACCOUNT_TYPE_EXPENSES: &str = "expenses";
pub enum AccountType {
    Assets,
    Liabilities,
    Equity,
    Revenues,
    Expenses,
}

impl AccountType {
    fn read(v: &str) -> Result<AccountType, ValidationError> {
        hledger::RE_ACCOUNT
            .captures(v)
            .and_then(|v| v.get(1))
            .and_then(|v| {
                let v = v.as_str();
                match v {
                    ACCOUNT_TYPE_ASSETS => Some(AccountType::Assets),
                    ACCOUNT_TYPE_LIABILITIES => Some(AccountType::Liabilities),
                    ACCOUNT_TYPE_EQUITY => Some(AccountType::Equity),
                    ACCOUNT_TYPE_REVENUES => Some(AccountType::Revenues),
                    ACCOUNT_TYPE_EXPENSES => Some(AccountType::Expenses),
                    _ => None,
                }
            })
            .ok_or({
                println!("invalid account name: {}", v);
                ValidationError::new("invalid account name")
            })
    }

    fn to_string<'a>(self) -> &'a str {
        match self {
            AccountType::Assets => ACCOUNT_TYPE_ASSETS,
            AccountType::Liabilities => ACCOUNT_TYPE_LIABILITIES,
            AccountType::Equity => ACCOUNT_TYPE_EQUITY,
            AccountType::Revenues => ACCOUNT_TYPE_REVENUES,
            AccountType::Expenses => ACCOUNT_TYPE_EXPENSES,
        }
    }

    fn must_not_exceed(self) -> (bool, bool) {
        let disallow_red = match self {
            AccountType::Assets | AccountType::Equity | AccountType::Expenses => true,
            AccountType::Revenues => false,
            AccountType::Liabilities => false,
        };
        let disallow_green = match self {
            AccountType::Assets | AccountType::Equity | AccountType::Expenses => false,
            AccountType::Revenues => true,
            AccountType::Liabilities => false,
        };
        (disallow_red, disallow_green)
    }
}
