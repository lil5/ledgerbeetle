use std::collections::HashMap;
use std::time::UNIX_EPOCH;
use std::{ops::Neg, sync::LazyLock};

use anyhow::anyhow;
use regex::Regex;
use serde::*;
use validator::Validate;

use crate::{models, tb_utils};

// Requests and Responses
// ------------------------------------
pub type RequestAdd = AddTransactions;
pub type ResponseAdd = Vec<String>;

pub type ResponseAccountNames = Vec<String>;
pub type ResponseCommodities = Vec<String>;
pub type ResponseTransactions = Vec<Transaction>;
pub type ResponseBalances = Vec<Balance>;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseIncomeStatements {
    pub dates: Vec<usize>,
    pub income_statements: Vec<IncomeStatement>,
}

// Types
// ------------------------------------

pub static RE_DATE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\d{4}-\d\d-\d\d$").expect("invalid regex"));
pub static RE_ACCOUNTS_FIND: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z0-9\*\.\|:]+$").expect("invalid regex"));
pub static RE_ACCOUNT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(assets|liabilities|equity|revenues|expenses):([a-z0-9]+:)*([a-z0-9]+)$")
        .expect("invalid regex")
});

#[derive(Default, Debug, Validate, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Default, Debug, Validate, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddTransactions {
    /// unix time milliseconds
    pub full_date2: i64,
    /// list of transactions
    #[validate(length(min = 1))]
    pub transactions: Vec<AddTransaction>,
}

#[derive(Default, Debug, Validate, Clone, PartialEq, Serialize, Deserialize)]
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

    // pub fn to_hledger_string(&self) -> String {
    //     let date = DateTime::<Utc>::from_timestamp(self.full_date, 0).unwrap();
    //     format!(
    //         "{} * transfer {} ; code {} related {}\n    {: >12} {: >10} {: <5}\n    {: >12} {: >10} {: <5}\n",
    //         date.to_rfc3339(),
    //         self.transfer_id,
    //         self.code,
    //         self.related_id,
    //         self.debit_account,
    //         self.debit_amount,
    //         self.commodity_unit,
    //         self.credit_account,
    //         self.credit_amount,
    //         self.commodity_unit,
    //     )
    // }
}

#[derive(Default, Debug, Validate, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    pub account_name: String,
    pub amount: i64,
    pub commodity_unit: String,
    pub commodity_decimal: i32,
}

#[derive(Default, Debug, Validate, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IncomeStatement {
    pub account_name: String,
    pub amounts: Vec<i64>,
    pub commodity_unit: String,
    pub commodity_decimal: i32,
}
