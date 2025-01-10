/// Rest api to read account data
/// https://docs.kraken.com/api/docs/category/rest-api/account-data
use crate::pair::Pair;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuySell {
    Buy,
    Sell,
}

/// Trade data from export csv file
#[derive(Serialize, Deserialize)]
pub struct TradesExport {
    /// Transaction ID inside of order
    pub txid: String,
    /// Order responsible for trade execution
    pub ordertxid: String,
    /// Asset pair
    pub pair: Pair,
    /// Unix timestamp of trade
    pub time: String,
    /// Type of order (buy/sell)
    #[serde(rename = "type")]
    pub typ: BuySell,
    /// Order type (market/limit)
    pub ordertype: String,
    /// price
    pub price: f32,
    /// Total cost of order (quote currency)
    pub cost: f32,
    /// Total fee (quote currency)
    pub fee: f32,
    /// Volume (base currency)
    pub vol: f32,
    /// Initial margin (quote currency)
    pub margin: f32,
    /// Comma delimited list of miscellaneous info
    pub misc: String,
    /// List of ledger ids for entries associated with trade
    pub ledgers: String,
    /// Total cost of order (USD)
    pub costusd: f32,
}

/// Trade data from REST API
#[derive(Serialize, Deserialize)]
pub struct TradesInfo {
    #[serde(default)]
    pub txid: String,
    /// Order responsible for trade execution
    pub ordertxid: String,
    /// Position responsible for execution of trade
    pub postxid: Option<String>,
    /// Asset pair
    pub pair: Pair,
    /// Unix timestamp of trade
    pub time: String,
    /// Type of order (buy/sell)
    #[serde(rename = "type")]
    pub typ: String,
    /// Order type
    pub ordertype: String,
    /// price
    pub price: f32,
    /// Total cost of order (quote currency)
    pub cost: f32,
    /// Total fee (quote currency)
    pub fee: f32,
    /// Volume (base currency)
    pub vol: f32,
    /// Initial margin (quote currency)
    pub margin: f32,
    /// Amount of leverage used in trade
    pub leverage: Option<f32>,
    /// Comma delimited list of miscellaneous info
    pub misc: String,
    /// List of ledger ids for entries associated with trade
    pub ledgers: Option<Vec<String>>,
    /// Unique identifier of trade executed
    pub trade_id: Option<u32>,
    /// True if trade was executed with user as the maker, false if take
    pub maker: Option<bool>,
    /// Total cost of order (USD)
    pub costusd: f32,
    /// See [`TradesInfoEx`]
    #[serde(flatten)]
    pub extra: Option<TradesInfoEx>,
}

/// Only present if Trade opened a position
#[derive(Serialize, Deserialize)]
pub struct TradesInfoEx {
    /// Position status (open/closed)
    posstatus: String,
    /// Average price of closed portion of position (quote currency)
    cprice: u64,
    /// Total cost of closed portion of position (quote currency)
    ccost: u64,
    /// Total fee of closed portion of position (quote currency)
    cfee: u64,
    /// Total volume of closed portion of position (quote currency)
    cvol: u64,
    /// Total margin freed in closed portion of position (quote currency)
    cmargin: u64,
    /// Net profit/loss of closed portion of position (quote currency, quote currency scale)
    net: u64,
    /// List of closing trades for position (if available)
    trades: Option<Vec<String>>,
}
