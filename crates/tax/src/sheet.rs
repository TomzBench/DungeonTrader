/// sheet.rs
///
/// See more about the configuration here:
/// https://github.com/eprbell/rp2/blob/main/docs/input_files.md#the-config-file
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::{collections::HashMap, str};

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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
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
        assert_eq!(11, config.in_header.fiat_fee);
        assert_eq!(9, config.out_header.crypto_fee);
        assert_eq!(None, config.out_header.notes);
        assert_eq!(Some(12), config.intra_header.notes);
        assert_eq!(
            Some(&AccountingMethod::Hifo),
            config.accounting_methods.unwrap().year.get(&2022)
        );
    }
}
