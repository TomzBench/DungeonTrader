/// Rest api to read account data
/// https://docs.kraken.com/api/docs/category/rest-api/account-data
use crate::pair::Pair;
use chrono::{DateTime, Utc};
use dt_plugin::{
    Input, InputTransaction, Output, OutputTransaction, Transaction, TransactionConfig,
    TransactionResult, Transactions,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuySell {
    Buy,
    Sell,
}

/// Trade data from export csv file
#[derive(Serialize, Deserialize)]
pub struct TradesExport<'a> {
    /// Transaction ID inside of order
    pub txid: &'a str,
    /// Order responsible for trade execution
    pub ordertxid: &'a str,
    /// Asset pair
    #[serde(borrow)]
    pub pair: Pair<'a>,
    /// Unix timestamp of trade
    #[serde(with = "dt_plugin::date")]
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
    #[serde(with = "dt_plugin::date")]
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

impl Transactions for TradesExport<'_> {
    /// Trades on Kraken against USD:
    /// - Cost = volume * price
    /// - Fee's are in USD
    fn transactions(&self, config: TransactionConfig) -> TransactionResult<Transaction> {
        match &self.pair.1 {
            x if *x == config.fiat => match self.typ {
                BuySell::Buy => {
                    let fiat_fee = self.fee * self.costusd / self.cost;
                    Ok(Transaction::Input(InputTransaction {
                        timestamp: self.time,
                        asset: &self.pair.0,
                        exchange: "kraken",
                        holder: "TODO",
                        typ: Input::Buy,
                        spot_price: self.price,
                        crypto_in: self.vol,
                        crypto_fee: 0.0,
                        fiat_in_no_fee: self.costusd, // vol * price
                        fiat_in_with_fee: self.costusd + fiat_fee,
                        fiat_fee,
                        unique_id: self.txid,
                        notes: self.ordertxid,
                    }))
                }
                BuySell::Sell => {
                    let fiat_fee = self.fee * self.costusd / self.cost;
                    Ok(Transaction::Output(OutputTransaction {
                        timestamp: self.time,
                        asset: &self.pair.0,
                        exchange: "kraken",
                        holder: "TODO",
                        typ: Output::Sell,
                        spot_price: self.price,
                        crypto_out_no_fee: self.vol,
                        crypto_out_with_fee: self.vol,
                        crypto_fee: 0.0,
                        fiat_out_no_fee: self.costusd,
                        fiat_fee,
                        unique_id: self.txid,
                        notes: self.ordertxid,
                    }))
                }
            },
            _ => match self.typ {
                BuySell::Buy => {
                    Ok(Transaction::Convert(
                        OutputTransaction {
                            timestamp: self.time,
                            asset: &self.pair.1,
                            exchange: "kraken",
                            // TODO known by caller
                            holder: "TODO",
                            typ: Output::Sell,
                            spot_price: self.costusd / self.cost,
                            crypto_out_no_fee: self.cost,
                            crypto_out_with_fee: self.cost + self.fee,
                            crypto_fee: self.fee,
                            fiat_out_no_fee: self.costusd,
                            fiat_fee: self.fee * self.costusd / self.cost,
                            unique_id: self.txid,
                            notes: self.ordertxid,
                        },
                        InputTransaction {
                            timestamp: self.time,
                            asset: &self.pair.0,
                            exchange: "kraken",
                            holder: "TODO",
                            typ: Input::Buy,
                            spot_price: (self.price * self.costusd) / self.cost,
                            crypto_in: self.vol,
                            crypto_fee: 0.0, // Captured on output side of transaction
                            fiat_in_no_fee: self.costusd,
                            fiat_in_with_fee: self.costusd,
                            fiat_fee: 0.0,
                            unique_id: self.txid,
                            notes: self.ordertxid,
                        },
                    ))
                }
                BuySell::Sell => unimplemented!(),
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::TradesExport;
    use dt_plugin::{Config, Transactions};
    use indoc::indoc;

    #[test]
    fn should_convert() {
        let input = indoc! {r#"
            "txid","ordertxid","pair","time","type","ordertype","price","cost","fee","vol","margin","misc","ledgers","costusd"
            "TD37HR-OG3BN-TZXXEN","OORD67-UN4NM-HTJP6Z","ETH/BTC","2016-04-13 18:51:09.3443","buy","market",0.019964,2.955151,0.007683,148.02400000,0.000000,"initiated","LBQUXD-OL6QO-DHOCCX,LYH3A5-FQVIP-NAD2KN",1255.17
        "#};

        let config = r#"
            [transactions]
            holder = "bob"
            fiat = "USD"
            "#;

        /*
        let data = csv::Reader::from_reader(&mut input.as_bytes())
            .deserialize()
            .collect::<csv::Result<Vec<TradesExport<'_>>>>()
            .unwrap()
            .pop()
            .unwrap();
        */

        let config: Config = toml::from_str(config).unwrap();

        //let tx = data.transactions(config.transactions).unwrap();
    }
}
