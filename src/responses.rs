use chrono::{DateTime, Utc};
use diesel::prelude::Insertable;
use diesel::Selectable;
use std::collections::HashMap;
use std::time::UNIX_EPOCH;
use std::{ops::Neg, sync::LazyLock};
use utoipa::ToSchema;

use anyhow::anyhow;
use regex::Regex;
use serde::*;
use validator::{Validate, ValidationError};

use crate::{models, tb_utils};

// Requests and Responses
// ------------------------------------
#[derive(Serialize, Deserialize, ToSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct RequestMigrate {
    #[validate(length(min = 1))]
    pub commodities: Vec<MigrateCommodity>,
    #[validate(length(min = 1))]
    pub accounts: Vec<MigrateAccount>,
}

pub type RequestAdd = AddTransactions;
pub type ResponseAdd = Vec<String>;

pub type RequestAddPrepareGlob = AddFilterTransactions;
pub type ResponseAddPrepare = RequestAdd;

pub type ResponseAccountNames = Vec<String>;
pub type ResponseCommodities = Vec<String>;
pub type ResponseTransactions = Vec<Transaction>;
pub type ResponseBalances = Vec<Balance>;

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResponseIncomeStatements {
    pub dates: Vec<usize>,
    pub income_statements: Vec<IncomeStatement>,
}

// Types
// ------------------------------------

pub static RE_ACCOUNTS_GLOB: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z0-9\*\.\|:]+$").expect("invalid regex"));
pub static RE_ACCOUNT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(a|l|e|r|x):([a-z0-9]+:)*([a-z0-9]+)$").expect("invalid regex"));

#[derive(Default, Debug, Validate, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MigrateAccount {
    /// tigerbeetle account id in hexadecimal
    pub t: String,
    /// account name
    pub n: String,
    /// tigerbeetle ledger id
    pub c: i64,
}

#[derive(
    Default,
    Debug,
    Validate,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    ToSchema,
    Selectable,
    Insertable,
)]
#[diesel(table_name = crate::schema::commodities)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[serde(rename_all = "camelCase")]
pub struct MigrateCommodity {
    /// tigerbeetle ledger number
    pub id: i32,
    /// commodity unit used
    pub unit: String,
    /// location of decimal point
    pub decimal_place: i32,
}

#[derive(Default, Debug, Validate, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    /// commodity used
    pub commodity_unit: String,
    /// location of decimal point
    pub commodity_decimal: i32,
    /// transaction code
    pub code: i32,
    /// unix time milliseconds
    pub full_date: i64,
    /// unit time milliseconds
    pub full_date2: i64,
    /// random hex u128 string
    pub related_id: String,
    /// random hex u128 string
    pub transfer_id: String,
    /// account name
    pub debit_account: String,
    /// account name
    pub credit_account: String,
    /// amount added to debit account
    pub debit_amount: i64,
    /// amount removed from credit account
    pub credit_amount: i64,
}

#[derive(Default, Debug, Validate, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddTransactions {
    /// unix time milliseconds
    pub full_date2: i64,
    /// list of transactions
    #[validate(length(min = 1))]
    pub transactions: Vec<AddTransaction>,
}

#[derive(Default, Debug, Validate, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddTransaction {
    /// commodity used
    pub commodity_unit: String,
    /// transaction code
    pub code: i32,
    /// random hex u128 string
    pub related_id: String,
    /// account name
    #[validate(regex(path=*RE_ACCOUNT))]
    pub debit_account: String,
    /// account name
    #[validate(regex(path=*RE_ACCOUNT))]
    pub credit_account: String,
    /// amount added to debit account
    #[validate(range(min = 1))]
    pub amount: i64,
}

impl AddTransactions {
    pub fn parse_from_csv_line(line: String) -> Result<AddTransactions, ValidationError> {
        let mut full_date2: i64 = 0;
        let mut transaction = AddTransaction {
            commodity_unit: String::new(),
            code: 0,
            related_id: String::new(),
            debit_account: String::new(),
            credit_account: String::new(),
            amount: 0,
        };

        for (i, v) in line.split(",").into_iter().enumerate() {
            match i {
                0 => {
                    transaction.commodity_unit = String::from(v);
                }
                2 => {
                    transaction.code = v
                        .parse::<i32>()
                        .map_err(|_| ValidationError::new("invalid full_date2"))?;
                }
                3 => {
                    full_date2 = v
                        .parse::<i64>()
                        .map_err(|_| ValidationError::new("invalid code"))?;
                }
                5 => {
                    transaction.related_id = String::from(v);
                }
                7 => {
                    transaction.debit_account = String::from(v);
                }
                8 => {
                    transaction.credit_account = String::from(v);
                }
                9 => {
                    transaction.amount = v
                        .parse::<i64>()
                        .map_err(|_| ValidationError::new("invalid debit_amount"))?;
                }
                _ => {}
            };
        }
        Ok(AddTransactions {
            full_date2,
            transactions: vec![transaction],
        })
    }
}

#[derive(Debug, Validate, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddFilterTransactions {
    /// unix time milliseconds
    pub full_date2: i64,
    /// list of transactions
    #[validate(length(min = 1))]
    pub filter_transactions: Vec<AddFilterTransaction>,
}

#[derive(Default, Debug, Validate, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddFilterTransaction {
    /// commodity used
    pub commodity_unit: String,
    /// transaction code
    pub code: i32,
    /// random hex u128 string
    pub related_id: String,
    /// account name
    #[validate(regex(path=*RE_ACCOUNT))]
    pub debit_account: String,
    /// account name
    #[validate(custom(
        function = "validate_add_filter_transaction_credit_accounts_filter",
        message = "invalid credit account filter"
    ))]
    pub credit_accounts_filter: Vec<String>,
    /// amount added to debit account
    #[validate(range(min = 1))]
    pub amount: i64,
}

fn validate_add_filter_transaction_credit_accounts_filter(
    credit_accounts_filter: &Vec<String>,
) -> Result<(), ValidationError> {
    if credit_accounts_filter
        .iter()
        .any(|v| !RE_ACCOUNTS_GLOB.is_match(v))
    {
        Err(ValidationError::new("invalid account filter"))
    } else {
        Ok(())
    }
}

impl Transaction {
    pub fn from_tb(
        transfer: tigerbeetle_unofficial::Transfer,
        accounts: HashMap<u128, &models::Account>,
        commodity: &&models::Commodities,
    ) -> Result<Transaction, anyhow::Error> {
        let date = (transfer.timestamp())
            .duration_since(UNIX_EPOCH)
            .map_err(|e| anyhow!(e))?
            .as_millis() as i64;
        let date2 = transfer.user_data_64() as i64;

        let debit_amount = transfer.amount() as i64;

        let debit_account = accounts.get(&transfer.debit_account_id());
        let credit_account = accounts.get(&transfer.credit_account_id());
        let debit_account = match debit_account {
            Some(v) => v.name.clone(),
            None => String::new(),
        };
        let credit_account = match credit_account {
            Some(v) => v.name.clone(),
            None => String::new(),
        };
        Ok(Transaction {
            code: transfer.code().into(),
            full_date: date,
            full_date2: date2,
            related_id: tb_utils::u128::to_hex_string(transfer.user_data_128()),
            transfer_id: tb_utils::u128::to_hex_string(transfer.id()),
            debit_account,
            credit_account,
            debit_amount,
            credit_amount: debit_amount.neg(),
            commodity_unit: commodity.unit.clone(),
            commodity_decimal: commodity.decimal_place,
        })
    }

    pub fn csv_header() -> &'static str {
        "commodity_unit,commodity_decimal,code,full_date,full_date2,related_id,transfer_id,debit_account,credit_account,debit_amount,credit_amount"
    }
    pub fn to_csv(&self) -> Result<String, ValidationError> {
        Ok(format!(
            "{},{},{},{},{},{},{},{},{},{},{}",
            self.commodity_unit,    //  0
            self.commodity_decimal, //  1
            self.code,              //  2
            self.full_date,         //  3
            self.full_date2,        //  4
            self.related_id,        //  5
            self.transfer_id,       //  6
            self.debit_account,     //  7
            self.credit_account,    //  8
            self.debit_amount,      //  9
            self.credit_amount,     // 10
        ))
    }
    pub fn to_hledger_string(&self) -> Result<String, ValidationError> {
        let date = DateTime::<Utc>::from_timestamp_millis(self.full_date)
            .ok_or(ValidationError::new("invalid full_date"))?;
        Ok(format!(
            "{} * {} ; related id {}, code {}\n    {: >12} {: >10} {: <5}\n    {: >12} {: >10} {: <5}\n",
            //line 1
            date.format("%Y-%m-%d"),
            self.transfer_id,
            self.related_id,
            self.code,
            //line 2
            self.debit_account,
            self.debit_amount,
            self.commodity_unit,
            //line 3
            self.credit_account,
            self.credit_amount,
            self.commodity_unit,
        ))
    }
}

#[derive(Default, Debug, Validate, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    pub account_name: String,
    pub amount: i64,
    pub commodity_unit: String,
    pub commodity_decimal: i32,
}

#[derive(Default, Debug, Validate, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct IncomeStatement {
    pub account_name: String,
    pub amounts: Vec<i64>,
    pub commodity_unit: String,
    pub commodity_decimal: i32,
}
