use crate::{hledger, http_err};
use axum_macros::debug_handler;
use deadpool_diesel::postgres::Object;
use diesel::{insert_into, prelude::*, result::Error::NotFound};
use regex::Regex;
use std::sync::{Arc, LazyLock};
use tigerbeetle_unofficial as tb;
use tokio::sync::{RwLock, RwLockReadGuard};
use validator::ValidationError;

pub static TB_MAX_BATCH_SIZE: u32 = 8190;

static RE_AMOUNT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d+)( ([A-Z]+))?$").unwrap());

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

    // return Err(http_err::internal_error(ValidationError::new("help")));
    match first_account {
        Ok(v) => {
            let currency = create_currency(conn, unit.clone()).await?;
            Ok((v, currency))
        }
        Err(err) => {
            if err != NotFound {
                Err(http_err::internal_error(err))
            } else {
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
    let account = conn
        .interact(move |conn| {
            let new_account = NewAccount {
                name: account_name.as_str(),
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

    let new_tb_account = tb::Account::new(
        account.tb_id.try_into().unwrap(),
        currency.tb_ledger.try_into().unwrap(),
        1,
    );
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
    use crate::schema::accounts::dsl;

    conn.interact(|conn| {
        dsl::accounts
            .select(Account::as_select())
            .filter(dsl::name.like(filter))
            .get_results::<Account>(conn)
            .map_err(http_err::internal_error)
    })
    .await
    .map_err(http_err::internal_error)?
}

pub fn read_amount(a: &str) -> Result<(i64, String), http_err::HttpErr> {
    let err = || http_err::bad_error(ValidationError::new("invalid amount"));
    let m = RE_AMOUNT.captures(a).ok_or(err())?;
    let one = m.get(1).ok_or(err()).map(|v| v.as_str())?;
    let one = one.parse::<i64>().map_err(|_| err())?;
    let unit = match m.get(3) {
        Some(v) => v.as_str(),
        None => "",
    }
    .to_string();

    Ok((one, unit))
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
        .map_err(http_err::internal_error)?;

    match first_currency {
        Ok(v) => Ok(v),
        Err(err) => {
            if err != NotFound {
                Err(http_err::internal_error(err))
            } else {
                create_currency(&conn, unit_clone).await
            }
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

static ACCOUNT_TYPE_ALIAS: &str = "s";
static ACCOUNT_TYPE_ASSETS: &str = "a";
static ACCOUNT_TYPE_LIABILITIES: &str = "l";
static ACCOUNT_TYPE_EQUITY: &str = "e";
static ACCOUNT_TYPE_REVENUES: &str = "r";
static ACCOUNT_TYPE_EXPENSES: &str = "x";
pub enum AccountType {
    Alias,
    Assets,
    Liabilities,
    Equity,
    Revenues,
    Expenses,
}

impl AccountType {
    fn read(v: &str) -> Result<AccountType, ValidationError> {
        let err = ValidationError::new("invalid account name");
        match v.get(0..1).ok_or(err.clone())? {
            "s" => Ok(AccountType::Alias),
            "a" => Ok(AccountType::Assets),
            "l" => Ok(AccountType::Liabilities),
            "e" => Ok(AccountType::Equity),
            "r" => Ok(AccountType::Revenues),
            "x" => Ok(AccountType::Expenses),
            _ => Err(err),
        }
    }

    fn to_string<'a>(self) -> &'a str {
        match self {
            AccountType::Alias => ACCOUNT_TYPE_ALIAS,
            AccountType::Assets => ACCOUNT_TYPE_ASSETS,
            AccountType::Liabilities => ACCOUNT_TYPE_LIABILITIES,
            AccountType::Equity => ACCOUNT_TYPE_EQUITY,
            AccountType::Revenues => ACCOUNT_TYPE_REVENUES,
            AccountType::Expenses => ACCOUNT_TYPE_EXPENSES,
        }
    }
}
