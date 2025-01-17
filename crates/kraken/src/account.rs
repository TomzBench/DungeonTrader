/// Rest api to read account data
/// https://docs.kraken.com/api/docs/category/rest-api/account-data
use chrono::{DateTime, Utc};
use dungeon_tax::Pair;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BuySell {
    Buy,
    Sell,
}

/// Trade data from export csv file
#[derive(Debug, Serialize, Deserialize)]
pub struct TradesExport<'a> {
    /// Transaction ID inside of order
    pub txid: &'a str,
    /// Order responsible for trade execution
    pub ordertxid: &'a str,
    /// Asset pair
    #[serde(borrow)]
    pub pair: Pair<'a>,
    /// Unix timestamp of trade
    #[serde(with = "dungeon_tax::date")]
    pub time: DateTime<Utc>,
    /// Type of order (buy/sell)
    #[serde(rename = "type")]
    pub typ: BuySell,
    /// Order type (market/limit)
    pub ordertype: &'a str,
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
    pub misc: &'a str,
    /// List of ledger ids for entries associated with trade
    pub ledgers: &'a str,
    /// Total cost of order (USD)
    pub costusd: f32,
}

/// Trade data from REST API
#[derive(Serialize, Deserialize)]
pub struct TradesInfo<'a> {
    #[serde(default)]
    pub txid: &'a str,
    /// Order responsible for trade execution
    pub ordertxid: &'a str,
    /// Position responsible for execution of trade
    pub postxid: Option<&'a str>,
    /// Asset pair
    #[serde(borrow)]
    pub pair: Pair<'a>,
    /// Unix timestamp of trade
    #[serde(with = "dungeon_tax::date")]
    pub time: DateTime<Utc>,
    /// Type of order (buy/sell)
    #[serde(rename = "type")]
    pub typ: &'a str,
    /// Order type
    pub ordertype: &'a str,
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
    pub misc: &'a str,
    /// List of ledger ids for entries associated with trade
    pub ledgers: Option<Vec<&'a str>>,
    /// Unique identifier of trade executed
    pub trade_id: Option<u32>,
    /// True if trade was executed with user as the maker, false if take
    pub maker: Option<bool>,
    /// Total cost of order (USD)
    pub costusd: f32,
    /// See [`TradesInfoEx`]
    #[serde(flatten)]
    pub extra: Option<TradesInfoEx<'a>>,
}

/// Only present if Trade opened a position
#[derive(Serialize, Deserialize)]
pub struct TradesInfoEx<'a> {
    /// Position status (open/closed)
    posstatus: &'a str,
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
    trades: Option<Vec<&'a str>>,
}
