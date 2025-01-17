/// sheet.rs
///
/// See more about the configuration here:
/// https://github.com/eprbell/rp2/blob/main/docs/input_files.md#the-config-file
use chrono::{DateTime, Utc};
use serde::{
    de::{self},
    Deserialize, Serialize,
};
use std::{collections::HashMap, fmt, io, str};

pub struct Visitor;
impl<'de> de::Visitor<'de> for Visitor {
    type Value = Vec<String>;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("expecting key = column value pairs")
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct General {
    pub assets: Vec<String>,
    pub exchanges: Vec<String>,
    pub holders: Vec<String>,
    pub spouse: Option<String>,
    pub generator: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}

macro_rules! impl_deserialize_header {
    ($name:ty) => {
        impl std::ops::Index<usize> for $name {
            type Output = String;
            fn index(&self, index: usize) -> &Self::Output {
                &self.0[index]
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(de: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                de.deserialize_map(Visitor).map(Self)
            }
        }
    };
}

// https://github.com/eprbell/rp2/blob/main/docs/input_files.md#in-transaction-table-format
#[derive(Debug)]
pub struct InputHeader(Vec<String>);
impl_deserialize_header!(InputHeader);

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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

// https://github.com/eprbell/rp2/blob/main/docs/input_files.md#out-transaction-table-format
#[derive(Debug)]
pub struct OutputHeader(Vec<String>);
impl_deserialize_header!(OutputHeader);

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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

// https://github.com/eprbell/rp2/blob/main/docs/input_files.md#intra-transaction-table-format
#[derive(Debug)]
pub struct IntraHeader(Vec<String>);
impl_deserialize_header!(IntraHeader);

#[derive(Debug, Serialize, Deserialize)]
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
pub struct Config {
    pub general: General,
    pub in_header: InputHeader,
    pub out_header: OutputHeader,
    pub intra_header: IntraHeader,
    pub accounting_methods: Option<AccountingMethods>,
}

impl Config {
    pub fn input_headers(&self) -> String {
        self.in_header.0.join(",")
    }

    pub fn output_headers(&self) -> String {
        self.out_header.0.join(",")
    }

    pub fn intra_headers(&self) -> String {
        self.intra_header.0.join(",")
    }
}

pub type AssetTables<W> = (csv::Writer<W>, csv::Writer<W>, csv::Writer<W>);
pub type AssetMap<'a, W> = HashMap<&'a str, AssetTables<W>>;

#[cfg(test)]
mod test {
    use super::{AccountingMethod, Config};
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
        let config = dungeon_ini::from_str::<Config>(input).unwrap();
        assert_eq!(4, config.general.assets.len());
        assert_eq!("B1", config.general.assets[0]);
        assert_eq!("B2", config.general.assets[1]);
        assert_eq!("B3", config.general.assets[2]);
        assert_eq!("B4", config.general.assets[3]);
        assert_eq!(
            Some("debug"),
            config
                .general
                .extra
                .get("meta")
                .as_ref()
                .map(|s| s.as_str())
        );
        assert_eq!("timestamp", config.in_header.0[0]);
        assert_eq!("asset", config.in_header[6]);
        assert_eq!("exchange", config.in_header[1]);
        assert_eq!("holder", config.in_header[2]);
        assert_eq!("transaction_type", config.in_header[5]);
        assert_eq!("spot_price", config.in_header[8]);
        assert_eq!("crypto_in", config.in_header[7]);
        assert_eq!("fiat_fee", config.in_header[11]);
        assert_eq!("fiat_in_no_fee", config.in_header[9]);
        assert_eq!("fiat_in_with_fee", config.in_header[10]);
        assert_eq!("notes", config.in_header[12]);
        assert_eq!("crypto_fee", config.out_header[9]);
        assert_eq!("notes", config.intra_header[12]);

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
        assert_eq!(expect_input.join(","), config.input_headers());
        assert_eq!(
            Some(&AccountingMethod::Hifo),
            config.accounting_methods.unwrap().year.get(&2022)
        );
    }
}
