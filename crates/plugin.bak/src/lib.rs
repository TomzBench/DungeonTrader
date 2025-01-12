pub mod config;
/// plugin
pub mod date;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{fmt, io};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(PartialEq))]
pub enum Input {
    Airdrop,
    Buy,
    Donate,
    Gift,
    Hardfork,
    Income,
    Interest,
    Mining,
    Staking,
    Wages,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(PartialEq))]
pub enum Output {
    Donate,
    Fee,
    Gift,
    Lost,
    Sell,
    Staking,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct InputTransaction<'a> {
    #[serde(with = "date")]
    pub timestamp: DateTime<Utc>,
    pub asset: &'a str,
    pub exchange: &'a str,
    pub holder: &'a str,
    #[serde(rename = "type")]
    pub typ: Input,
    pub spot_price: f32,
    pub crypto_in: f32,
    pub crypto_fee: f32,
    pub fiat_in_no_fee: f32,
    pub fiat_in_with_fee: f32,
    pub fiat_fee: f32,
    pub unique_id: &'a str,
    pub notes: &'a str,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct OutputTransaction<'a> {
    #[serde(with = "date")]
    pub timestamp: DateTime<Utc>,
    pub asset: &'a str,
    pub exchange: &'a str,
    pub holder: &'a str,
    pub typ: Output,
    pub spot_price: f32,
    pub crypto_out_no_fee: f32,
    pub crypto_fee: f32,
    pub crypto_out_with_fee: f32,
    pub fiat_out_no_fee: f32,
    pub fiat_fee: f32,
    pub unique_id: &'a str,
    pub notes: &'a str,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct IntraTransaction<'a> {
    #[serde(with = "date")]
    pub timestamp: DateTime<Utc>,
    pub asset: &'a str,
    pub from_exchange: &'a str,
    pub from_holder: &'a str,
    pub to_exchange: &'a str,
    pub to_holder: &'a str,
    pub spot_price: f32,
    pub crypto_sent: f32,
    pub crypto_received: f32,
    pub unique_id: &'a str,
    pub notes: &'a str,
}

#[derive(Deserialize)]
pub struct TransactionConfig {
    /// Slave (IE: Bob)
    pub holder: String,
    /// Slave money to pay your overload (IE: USD)
    pub fiat: String,
}

#[derive(Deserialize)]
pub struct Config {
    pub transactions: TransactionConfig,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(PartialEq))]
pub enum Transaction<'a> {
    Input(#[serde(borrow)] InputTransaction<'a>),
    Output(#[serde(borrow)] OutputTransaction<'a>),
    Intra(#[serde(borrow)] IntraTransaction<'a>),
    Convert(#[serde(borrow)] OutputTransaction<'a>, InputTransaction<'a>),
}

#[derive(Debug)]
pub enum TransactionError {}
impl std::error::Error for TransactionError {}
impl fmt::Display for TransactionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "")
    }
}

pub struct TransactionTable<W: io::Write> {
    buy: csv::Writer<W>,
    sell: csv::Writer<W>,
    transfer: csv::Writer<W>,
}

pub type TransactionResult<T> = std::result::Result<T, TransactionError>;

pub trait Transactions {
    fn transactions(&self, config: TransactionConfig) -> TransactionResult<Transaction>;
}
