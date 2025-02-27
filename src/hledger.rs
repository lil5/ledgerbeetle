use itertools::Itertools as _;
use std::borrow::Cow;
use std::collections::HashMap;
use std::time::UNIX_EPOCH;
use std::{ops::Neg, sync::LazyLock};
use tigerbeetle_unofficial as tb;

use anyhow::anyhow;
use chrono::DateTime;
use regex::Regex;
use serde::*;
use validator::{Validate, ValidationError};

use crate::{models, tb_utils};

// Requests and Responses
// ------------------------------------
pub type RequestAdd = Transaction;
// pub type RequestAddSingular = Transaction;
// pub type RequestAddMultiple = Vec<Transaction>;

// #[derive(Deserialize)]
// #[serde(untagged)]
// pub enum RequestAdd {
//     RequestAddSingular(RequestAddSingular),
//      RequestAddMultiple(RequestAddMultiple),
// }

pub type ResponseAccountNames = Vec<String>;
pub type ResponseAccountTransactions = Vec<(
    Transaction,
    Transaction,
    bool,
    String,
    Vec<Amount>,
    Vec<Amount>,
)>;
pub type ResponseCommodities = Vec<String>;
pub type ResponseTransactions = Vec<Transaction>;

// Types
// ------------------------------------

pub type Value = Option<()>;
pub static RE_DATE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d{4}-\d\d-\d\d$").unwrap());

#[derive(Default, Debug, Validate, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub tcode: String,
    pub tcomment: String,
    #[validate(regex(path=*RE_DATE))]
    pub tdate: String,
    pub tfulldate: Option<i64>,
    pub tdate2: Option<String>,
    pub tfulldate2: Option<i64>,
    pub tdescription: String,
    pub tindex: i64,
    #[validate(custom(function = "validate_postings"))]
    pub tpostings: Vec<Posting>,
    pub tprecedingcomment: String,
    pub tsourcepos: Vec<SourcePos>,
    pub tstatus: String,
    pub ttags: Vec<Value>,
}

fn validate_postings(postings: &Vec<Posting>) -> Result<(), ValidationError> {
    if postings.len() % 2 != 0 {
        return Err(ValidationError::new("postings must devisable by 2"));
    }
    match postings.iter().find_map(|posting| posting.validate().err()) {
        Some(err) => {
            if !err.is_empty() {
                let message = err.errors().iter().nth(0).unwrap().0.to_string();
                let err: ValidationError = ValidationError {
                    code: Cow::Owned(message),
                    message: None,
                    params: HashMap::new(),
                };
                return Err(err);
            }
        }
        None => {}
    }

    for chunk in &postings.into_iter().chunks(2) {
        let v = chunk.collect::<Vec<&Posting>>();
        let posting_credit = v.get(0).unwrap();
        let posting_debit = v.get(0).unwrap();

        posting_credit.validate_against_debit(posting_debit)?;
    }

    Ok(())
}

impl Transaction {
    pub fn from_tb(
        transfer: tigerbeetle_unofficial::Transfer,
        accounts: Box<HashMap<u128, &models::Account>>,
        currency: &&models::Currencies,
        amount_style: AmountStyle,
        index: &mut i64,
    ) -> Result<Transaction, anyhow::Error> {
        let tdate = DateTime::from_timestamp(
            (transfer.timestamp())
                .duration_since(UNIX_EPOCH)
                .map_err(|e| anyhow!(e))?
                .as_secs() as i64,
            0,
        )
        .ok_or(anyhow!("tigerbeetle should return a timestamp"))?;
        let tdate2 = DateTime::from_timestamp(transfer.user_data_64() as i64, 0);

        let postings = vec![transfer.credit_account_id(), transfer.debit_account_id()]
            .iter()
            .enumerate()
            .map(|(index, account_id)| {
                let amount = {
                    let amount = transfer.amount() as i64;
                    if index == 0 {
                        amount.neg()
                    } else {
                        amount
                    }
                };
                let amount = Amount::from_tb(amount, currency.unit.clone(), amount_style.clone());
                Posting::from_tb(
                    accounts
                        .get(account_id)
                        .and_then(|v| Some(v.name.clone()))
                        .unwrap_or(String::new()),
                    amount,
                    transfer.id().to_string(),
                )
            })
            .collect();

        Ok(Transaction {
            tcode: transfer.code().to_string(),
            tcomment: String::new(),
            tdate: tdate.format("%Y-%m-%d").to_string(),
            tfulldate: Some(tdate.timestamp()),
            tdate2: tdate2.and_then(|v| Some(v.format("%Y-%m-%d").to_string())),
            tfulldate2: tdate2.and_then(|v| Some(v.timestamp())),
            tdescription: tb_utils::u128::to_hex_string(transfer.user_data_128()),
            tindex: *index,
            tpostings: postings,
            tprecedingcomment: String::new(),
            tsourcepos: vec![],
            tstatus: "".to_string(),
            ttags: vec![],
        })
    }
}

#[derive(Default, Debug, Validate, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Posting {
    #[validate(length(min = 1))]
    pub paccount: String,
    #[validate(length(min = 1))]
    pub pamount: Vec<Amount>,
    pub pbalanceassertion: Value,
    pub pcomment: String,
    pub pdate: Value,
    pub pdate2: Value,
    pub poriginal: Value,
    pub pstatus: String,
    pub ptags: Vec<Value>,
    /// both credit and debit transactions should have the same ptransaction_ value
    #[serde(rename = "ptransaction_")]
    pub ptransaction: String,
    pub ptype: String,
}

impl Posting {
    pub fn from_tb(paccount: String, pamount: Amount, ptransaction: String) -> Posting {
        Posting {
            paccount: paccount,
            pamount: vec![pamount],
            pbalanceassertion: None,
            pcomment: "".to_string(),
            pdate: None,
            pdate2: None,
            poriginal: None,
            pstatus: "Cleared".to_string(),
            ptags: vec![],
            ptransaction: ptransaction,
            ptype: "RegularPosting".to_string(),
        }
    }

    pub fn validate_against_debit(&self, debit: &Posting) -> Result<(), ValidationError> {
        Ok(())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Amount {
    pub acommodity: String,
    pub acost: Value,
    pub aquantity: Quantity,
    pub astyle: AmountStyle,
}

/// Style of the commodity in use
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmountStyle {
    pub ascommodityside: String,
    pub ascommodityspaced: bool,
    pub asdecimalmark: String,
    pub asdigitgroups: Value,
    pub asprecision: i64,
    pub asrounding: String,
}
impl AmountStyle {
    pub fn from_tb(commodity: String) -> AmountStyle {
        if commodity == "" {
            AmountStyle {
                ascommodityside: "L".to_string(),
                ascommodityspaced: false,
                asdecimalmark: ".".to_string(),
                asdigitgroups: None,
                asprecision: 0,
                asrounding: "NoRounding".to_string(),
            }
        } else {
            AmountStyle {
                ascommodityside: "R".to_string(),
                ascommodityspaced: true,
                asdecimalmark: ".".to_string(),
                asdigitgroups: None,
                asprecision: 0,
                asrounding: "NoRounding".to_string(),
            }
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Quantity {
    pub decimal_mantissa: i64,
    pub decimal_places: i64,
    pub floating_point: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourcePos {
    pub source_column: i64,
    pub source_line: i64,
    pub source_name: String,
}

impl Quantity {
    pub fn from_tb(amount: i64) -> Quantity {
        Quantity {
            decimal_mantissa: amount,
            decimal_places: 0,
            floating_point: amount,
        }
    }
}

impl Amount {
    pub fn from_tb(amount: i64, currency_unit: String, astyle: AmountStyle) -> Amount {
        Amount {
            acommodity: currency_unit,
            acost: None,
            aquantity: Quantity::from_tb(amount),
            astyle: astyle,
        }
    }
}
