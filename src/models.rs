use deadpool_diesel::postgres::Object;
use diesel::{prelude::*, result::Error::NotFound};
use regex::Regex;
use std::sync::{Arc, LazyLock};
use tigerbeetle_unofficial as tb;
use validator::ValidationError;

static RE_AMOUNT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d+)( ([A-Z]+))?$").unwrap());

use crate::{bad_error, internal_error, HttpErr};

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::accounts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Account {
    pub id: i64,
    pub name: String,
    pub tb_id: i64,
}

pub async fn list_all_accounts(conn: &Object) -> Result<Vec<String>, HttpErr> {
    use crate::schema::accounts::dsl::*;
    conn.interact(|conn| {
        return accounts
            .select(name)
            .load::<String>(conn)
            .map_err(internal_error);
    })
    .await
    .map_err(internal_error)?
}

pub async fn find_or_create_account(
    conn: &Object,
    tb: &Arc<tb::Client>,
    account_name: String,
    unit: String,
) -> Result<Account, HttpErr> {
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
        .map_err(internal_error)?;

    match first_account {
        Ok(v) => Ok(v),
        Err(err) => {
            if err != NotFound {
                return Err(internal_error(err));
            } else {
                return create_account(&conn, &tb, account_name_clone, unit).await;
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
    tb: &Arc<tb::Client>,
    account_name: String,
    unit: String,
) -> Result<Account, HttpErr> {
    let currency = find_or_create_currency(&conn, unit).await?;
    conn.interact(move |conn| {
        let new_account = NewAccount {
            name: account_name.as_str(),
            currencies_id: &currency.id,
        };
        diesel::insert_into(crate::schema::accounts::table)
            .values(&new_account)
            .returning(Account::as_returning())
            .get_result(conn)
            .map_err(internal_error)
    })
    .await
    .map_err(internal_error)?
}

pub fn read_amount(a: &str) -> Result<(i64, String), HttpErr> {
    let err = || bad_error(ValidationError::new("invalid amount"));
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

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::transfer_details)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransferDetail {
    pub id: i64,
    pub tb_id: i64,
    pub description: String,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::currencies)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Currencies {
    pub id: i32,
    pub tb_ledger: i32,
    pub unit: String,
}

async fn find_or_create_currency(conn: &Object, unit: String) -> Result<Currencies, HttpErr> {
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
        .map_err(internal_error)?;

    match first_currency {
        Ok(v) => Ok(v),
        Err(err) => {
            if err != NotFound {
                Err(internal_error(err))
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
async fn create_currency(conn: &Object, unit: String) -> Result<Currencies, HttpErr> {
    conn.interact(move |conn| {
        let new_currency = NewCurrency {
            unit: unit.as_str(),
        };
        return diesel::insert_into(crate::schema::currencies::table)
            .values(&new_currency)
            .returning(Currencies::as_returning())
            .get_result(conn)
            .map_err(internal_error);
    })
    .await
    .map_err(internal_error)?
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
