/// sheet.rs
///
/// See more about the configuration here:
/// https://github.com/eprbell/rp2/blob/main/docs/input_files.md#the-config-file
use chrono::{DateTime, Utc};
use serde::{
    de::{self},
    Deserialize,
};
use std::{collections::HashMap, fmt, str};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct General<'a> {
    pub assets: Vec<&'a str>,
    pub exchanges: Vec<&'a str>,
    pub holders: Vec<&'a str>,
    pub spouse: Option<&'a str>,
    pub generator: Option<&'a str>,
    #[serde(flatten, borrow)]
    pub extra: HashMap<&'a str, &'a str>,
}

#[derive(Deserialize)]
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

/*
pub struct InputHeader {
    pub timestamp: u8,
    pub asset: u8,
    pub exchange: u8,
    pub holder: u8,
    pub transaction_type: u8,
    pub spot_price: u8,
    pub crypto_in: u8,
    pub crypto_fee: Option<u8>,
    pub fiat_fee: u8,
    pub fiat_in_no_fee: Option<u8>,
    pub fiat_in_with_fee: Option<u8>,
    pub notes: Option<u8>,
}
*/

#[derive(Debug)]
pub struct InputHeader(Vec<String>);
impl<'de> Deserialize<'de> for InputHeader {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_map(Visitor).map(Self)
    }
}

#[derive(Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct InputData<'a> {
    #[serde(with = "crate::date")]
    pub timestamp: DateTime<Utc>,
    pub asset: &'a str,
    pub exchange: &'a str,
    pub holder: &'a str,
    #[serde(rename = "type")]
    pub typ: Input,
    pub spot_price: f32,
    pub crypto_in: f32,
    pub crypto_fee: Option<f32>,
    pub fiat_in_no_fee: Option<f32>,
    pub fiat_in_with_fee: Option<f32>,
    pub fiat_fee: f32,
    pub unique_id: &'a str,
    pub notes: Option<&'a str>,
}

#[derive(Deserialize)]
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OutputHeader {
    pub timestamp: u8,
    pub asset: u8,
    pub exchange: u8,
    pub holder: u8,
    pub transaction_type: u8,
    pub spot_price: u8,
    pub crypto_out_no_fee: u8,
    pub crypto_fee: u8,
    pub crypto_out_with_fee: Option<u8>,
    pub fiat_out_no_fee: Option<u8>,
    pub fiat_fee: Option<u8>,
    pub notes: Option<u8>,
}

#[derive(Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct OutputData<'a> {
    #[serde(with = "crate::date")]
    pub timestamp: DateTime<Utc>,
    pub asset: &'a str,
    pub exchange: &'a str,
    pub holder: &'a str,
    pub typ: Output,
    pub spot_price: f32,
    pub crypto_out_no_fee: f32,
    pub crypto_fee: f32,
    pub crypto_out_with_fee: Option<f32>,
    pub fiat_out_no_fee: Option<f32>,
    pub fiat_fee: Option<f32>,
    pub unique_id: &'a str,
    pub notes: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct IntraHeader {
    pub timestamp: u8,
    pub asset: u8,
    pub from_exchange: u8,
    pub from_holder: u8,
    pub to_exchange: u8,
    pub to_holder: u8,
    pub crypto_sent: u8,
    pub crypto_received: u8,
    pub spot_price: Option<u8>,
    pub notes: Option<u8>,
}

#[derive(Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct IntraData<'a> {
    #[serde(with = "crate::date")]
    pub timestamp: DateTime<Utc>,
    pub asset: &'a str,
    pub from_exchange: &'a str,
    pub from_holder: &'a str,
    pub to_exchange: &'a str,
    pub to_holder: &'a str,
    pub spot_price: Option<f32>,
    pub crypto_sent: f32,
    pub crypto_received: f32,
    pub unique_id: &'a str,
    pub notes: Option<&'a str>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(PartialEq))]
pub enum Entry<'a> {
    Input(#[serde(borrow)] InputData<'a>),
    Output(#[serde(borrow)] OutputData<'a>),
    Intra(#[serde(borrow)] IntraData<'a>),
}

#[derive(Debug, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum AccountingMethod {
    Fifo,
    Lifo,
    Hifo,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AccountingMethods {
    #[serde(flatten)]
    pub year: HashMap<u16, AccountingMethod>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Headers<'a> {
    #[serde(borrow)]
    pub general: General<'a>,
    pub in_header: InputHeader,
    pub out_header: OutputHeader,
    pub intra_header: IntraHeader,
    pub accounting_methods: Option<AccountingMethods>,
}

impl<'a> Headers<'a> {
    //"txid","ordertxid","pair","time","type","ordertype","price","cost","fee","vol","margin","misc","ledgers","costusd"

    pub fn input(&self) -> String {
        self.in_header.0.join(",")
    }

    pub fn output(&self) -> String {
        unimplemented!()
    }

    pub fn intra(&self) -> String {
        // TODO output a sorted intra csv header
        unimplemented!()
    }
}

/// A data source which can populate a vector of transactions
pub trait Importer {
    fn import(&self, entries: &mut Vec<Entry>, config: Headers);

    fn size_hint(&self) -> usize;
}

pub struct Visitor;
impl<'de> de::Visitor<'de> for Visitor {
    type Value = Vec<String>;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("expecting key = idx value pairs")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut inner = Vec::new();
        while let Some((key, value)) = map.next_entry::<&'de str, usize>()? {
            if value >= inner.len() {
                for _ in inner.len()..value + 1 {
                    inner.push("".to_string());
                }
            }
            inner[value] = key.to_owned();
        }
        Ok(inner)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(v.split(",").map(|s| s.to_owned()).collect())
    }
}

#[cfg(test)]
mod test {
    use super::{AccountingMethod, Headers};
    use indoc::indoc;
    #[test]
    fn should_parse_config() {
        let input = indoc! {r#"
            [general]
            assets = B1, B2, B3, B4
            exchanges = BlockFi, Coinbase, Coinbase Pro, Kraken
            holders = Bob, Alice
            meta = debug

            [in_header]
            timestamp = 0
            asset = 6
            exchange = 1
            holder = 2
            transaction_type = 5
            spot_price = 8
            crypto_in = 7
            fiat_fee = 11
            fiat_in_no_fee = 9
            fiat_in_with_fee = 10
            notes = 12
            
            [out_header]
            timestamp = 0
            asset = 6
            exchange = 1
            holder = 2
            transaction_type = 5
            spot_price = 8
            crypto_out_no_fee = 7
            crypto_fee = 9
            
            [intra_header]
            timestamp = 0
            asset = 6
            from_exchange = 1
            from_holder = 2
            to_exchange = 3
            to_holder = 4
            spot_price = 8
            crypto_sent = 7
            crypto_received = 10
            notes = 12

            [accounting_methods]
            2020 = fifo
            2021 = lifo
            2022 = hifo
            2023 = fifo

        "#};
        let config = dungeon_ini::from_str::<Headers>(input).unwrap();
        assert_eq!(4, config.general.assets.len());
        assert_eq!("B1", config.general.assets[0]);
        assert_eq!("B2", config.general.assets[1]);
        assert_eq!("B3", config.general.assets[2]);
        assert_eq!("B4", config.general.assets[3]);
        assert_eq!(Some(&"debug"), config.general.extra.get("meta"));
        assert_eq!("timestamp", config.in_header.0[0]);
        assert_eq!("asset", config.in_header.0[6]);
        assert_eq!("exchange", config.in_header.0[1]);
        assert_eq!("holder", config.in_header.0[2]);
        assert_eq!("transaction_type", config.in_header.0[5]);
        assert_eq!("spot_price", config.in_header.0[8]);
        assert_eq!("crypto_in", config.in_header.0[7]);
        assert_eq!("fiat_fee", config.in_header.0[11]);
        assert_eq!("fiat_in_no_fee", config.in_header.0[9]);
        assert_eq!("fiat_in_with_fee", config.in_header.0[10]);
        assert_eq!("notes", config.in_header.0[12]);
        assert_eq!(9, config.out_header.crypto_fee);
        assert_eq!(None, config.out_header.notes);
        assert_eq!(Some(12), config.intra_header.notes);

        let expect_input = [
            "timestamp",
            "exchange",
            "holder",
            "",
            "",
            "transaction_type",
            "asset",
            "crypto_in",
            "spot_price",
            "fiat_in_no_fee",
            "fiat_in_with_fee",
            "fiat_fee",
            "notes",
        ];
        assert_eq!(expect_input.join(","), config.input());
        assert_eq!(
            Some(&AccountingMethod::Hifo),
            config.accounting_methods.unwrap().year.get(&2022)
        );
    }
}
