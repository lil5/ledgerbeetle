use crate::{http_err, responses, tb_utils::u128};
use deadpool_diesel::postgres::Object;
use diesel::{prelude::*, result::Error::NotFound};

use std::sync::Arc;
use tigerbeetle_unofficial as tb;
use tokio::sync::RwLock;
use validator::ValidationError;

pub static TB_MAX_BATCH_SIZE: u32 = 8190;

// #[derive(Queryable, Selectable)]
// #[diesel(table_name = crate::schema::commodities)]
// #[diesel(check_for_backend(diesel::pg::Pg))]
// pub struct CommoditiesInsertItem {
//     pub name: String,
//     pub tb_id: String,
//     pub commodities_id: i32,
// }

#[derive(Selectable, Queryable)]
#[diesel(table_name = crate::schema::commodities)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CommodityInsertResponse {
    pub id: i32,
    pub tb_ledger: i32,
}

pub async fn insert_commodities(
    conn: &Object,
    new_commodities: Vec<responses::MigrateCommodity>,
) -> http_err::HttpResult<Vec<CommodityInsertResponse>> {
    use crate::schema::commodities::dsl::*;
    let tb_ledger_filter: Vec<i32> = new_commodities.iter().map(|c| c.tb_ledger).collect();

    conn.interact(move |conn| {
        diesel::insert_into(commodities)
            .values(&new_commodities)
            .on_conflict_do_nothing()
            .execute(conn)
            .map_err(http_err::internal_error)
    })
    .await
    .map_err(http_err::internal_error)??;

    conn.interact(move |conn| {
        commodities
            .select(CommodityInsertResponse::as_select())
            .filter(tb_ledger.eq_any(tb_ledger_filter))
            .get_results::<CommodityInsertResponse>(conn)
            .map_err(http_err::internal_error)
    })
    .await
    .map_err(http_err::internal_error)?
}

pub async fn insert_accounts(
    conn: &Object,
    new_accounts: Vec<Account>,
) -> http_err::HttpResult<()> {
    use crate::schema::accounts::dsl::*;

    conn.interact(move |conn| {
        diesel::insert_into(accounts)
            .values(&new_accounts)
            .execute(conn)
            .map_err(http_err::internal_error)
    })
    .await
    .map_err(http_err::internal_error)??;

    Ok(())
}

#[derive(Queryable, Selectable, Insertable, Clone)]
#[diesel(table_name = crate::schema::accounts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Account {
    pub name: String,
    pub tb_id: String,
    pub commodities_id: i32,
}

pub async fn list_all_accounts(conn: &Object) -> Result<Vec<String>, http_err::HttpErr> {
    use crate::schema::accounts::dsl::*;
    conn.interact(|conn| {
        accounts
            .distinct()
            .select(name)
            .order(name)
            .load::<String>(conn)
            .map_err(http_err::internal_error)
    })
    .await
    .map_err(http_err::internal_error)?
}

pub async fn find_or_create_account(
    tb: Arc<RwLock<tb::Client>>,
    conn: &Object,
    account_name: String,
    unit: String,
) -> http_err::HttpResult<(Account, Commodities)> {
    use crate::schema::accounts::dsl::*;

    let account_name_clone = account_name.clone();
    let commodity = find_or_create_commodity(conn, unit.clone()).await?;
    let commodity_id_clone = commodity.id;
    let first_account = conn
        .interact(move |conn| {
            accounts
                .select(Account::as_select())
                .filter(name.eq(account_name))
                .filter(commodities_id.eq(commodity_id_clone))
                .first(conn)
        })
        .await
        .map_err(http_err::internal_error)?;

    match first_account {
        Ok(v) => Ok((v, commodity)),
        Err(err) => {
            if err != NotFound {
                Err(http_err::internal_error(err))
            } else {
                // println!("account {} not found creating...", account_name_clone);
                create_account(conn, tb, account_name_clone, unit.clone()).await
            }
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name =  crate::schema::accounts)]
pub struct NewAccount<'a> {
    pub name: &'a str,
    pub commodities_id: &'a i32,
    pub tb_id: &'a str,
}
async fn create_account(
    conn: &Object,
    tb: Arc<RwLock<tb::Client>>,
    account_name: String,
    unit: String,
) -> Result<(Account, Commodities), http_err::HttpErr> {
    // return Err(http_err::internal_error(ValidationError::new("stuff")));
    let commodity = find_or_create_commodity(conn, unit).await?;
    let account_name_clone = account_name.clone();
    let account_type = AccountType::read(account_name.as_str()).map_err(http_err::bad_error)?;
    let id = tb::id();
    // println!("creating account_name: {}", account_name);
    let account = conn
        .interact(move |conn| {
            let new_account = NewAccount {
                name: account_name_clone.as_str(),
                commodities_id: &commodity.id,
                tb_id: &u128::to_hex_string(id),
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
        let mut flags = tb::account::Flags::HISTORY;
        let (disallow_red, disallow_green) = account_type.must_not_exceed();
        if disallow_green {
            flags |= tb::account::Flags::DEBITS_MUST_NOT_EXCEED_CREDITS
        }
        if disallow_red {
            flags |= tb::account::Flags::CREDITS_MUST_NOT_EXCEED_DEBITS
        }
        flags
    };

    let new_tb_account = tb::Account::new(
        id,
        commodity
            .tb_ledger
            .try_into()
            .expect("tb_ledger should be able to become unsigned"),
        1,
    )
    .with_flags(flags);

    tb.read()
        .await
        .create_accounts(vec![new_tb_account])
        .await
        .map_err(http_err::internal_error)?;

    Ok((account, commodity))
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
        let q = q.select(Account::as_select()).order((name, commodities_id));
        q.get_results::<Account>(conn)
            .map_err(http_err::internal_error)
    })
    .await
    .map_err(http_err::internal_error)?
}

pub async fn find_accounts_re_by_commodity(
    conn: &Object,
    filter: String,
    commodity_unit: String,
) -> Result<Vec<Account>, http_err::HttpErr> {
    use crate::schema::accounts::dsl::*;
    use crate::schema::commodities::dsl::{id as commodities_table_id, *};

    conn.interact(move |conn| {
        let filter = filter.replace("**", "%").replace("*", "_");
        let mut q = accounts
            .inner_join(
                commodities.on(commodities_table_id
                    .eq(commodities_id)
                    .and(unit.eq(commodity_unit))),
            )
            .into_boxed();
        for (i, f) in filter.split("|").enumerate() {
            if i == 0 {
                q = q.filter(name.like(f));
            } else {
                q = q.or_filter(name.like(f));
            }
        }
        let q = q.select(Account::as_select()).order((name, commodities_id));
        q.get_results::<Account>(conn)
            .map_err(http_err::internal_error)
    })
    .await
    .map_err(http_err::internal_error)?
}

pub async fn find_accounts_by_tb_ids(
    conn: &Object,
    tb_ids: Vec<String>,
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
#[diesel(table_name = crate::schema::commodities)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Commodities {
    pub id: i32,
    pub tb_ledger: i32,
    pub unit: String,
    pub decimal_place: i32,
}

async fn find_or_create_commodity(
    conn: &Object,
    commodity_unit: String,
) -> Result<Commodities, http_err::HttpErr> {
    use crate::schema::commodities::dsl::*;
    let unit_clone = commodity_unit.clone();
    let first_commodity = conn
        .interact(|conn| {
            commodities
                .select(Commodities::as_select())
                .filter(unit.eq(commodity_unit))
                .first(conn)
        })
        .await
        .map_err(http_err::teapot_error)?;

    match first_commodity {
        Ok(v) => Ok(v),
        Err(err) => {
            if err != NotFound {
                println!("{}", err);
                Err(http_err::internal_error(err))
            } else {
                create_commodity(conn, unit_clone).await
            }
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name =  crate::schema::commodities)]
pub struct Newcommodity<'a> {
    pub unit: &'a str,
}
async fn create_commodity(conn: &Object, unit: String) -> Result<Commodities, http_err::HttpErr> {
    conn.interact(move |conn| {
        let new_commodity = Newcommodity {
            unit: unit.as_str(),
        };
        diesel::insert_into(crate::schema::commodities::table)
            .values(&new_commodity)
            .returning(Commodities::as_returning())
            .get_result(conn)
            .map_err(http_err::internal_error)
    })
    .await
    .map_err(http_err::internal_error)?
}

pub async fn list_all_commodity_units(conn: &Object) -> Result<Vec<String>, http_err::HttpErr> {
    conn.interact(move |conn| {
        use crate::schema::commodities::dsl::*;
        commodities
            .select(unit)
            .order(unit)
            .load(conn)
            .map_err(http_err::internal_error)
    })
    .await
    .map_err(http_err::internal_error)?
}
pub async fn list_all_commodities(conn: &Object) -> Result<Vec<Commodities>, http_err::HttpErr> {
    conn.interact(move |conn| {
        use crate::schema::commodities::dsl::*;
        commodities
            .select(commodities::all_columns())
            .load(conn)
            .map_err(http_err::internal_error)
    })
    .await
    .map_err(http_err::internal_error)?
}

const ACCOUNT_TYPE_ASSETS: &str = "a";
const ACCOUNT_TYPE_LIABILITIES: &str = "l";
const ACCOUNT_TYPE_EQUITY: &str = "e";
const ACCOUNT_TYPE_REVENUES: &str = "r";
const ACCOUNT_TYPE_EXPENSES: &str = "x";
pub enum AccountType {
    Assets,
    Liabilities,
    Equity,
    Revenues,
    Expenses,
}

impl AccountType {
    fn read(v: &str) -> Result<AccountType, ValidationError> {
        responses::RE_ACCOUNT
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

    // fn to_string<'a>(self) -> &'a str {
    //     match self {
    //         AccountType::Assets => ACCOUNT_TYPE_ASSETS,
    //         AccountType::Liabilities => ACCOUNT_TYPE_LIABILITIES,
    //         AccountType::Equity => ACCOUNT_TYPE_EQUITY,
    //         AccountType::Revenues => ACCOUNT_TYPE_REVENUES,
    //         AccountType::Expenses => ACCOUNT_TYPE_EXPENSES,
    //     }
    // }

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
