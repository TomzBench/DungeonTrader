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
use tracing::warn;

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

macro_rules! impl_xlsx_writer {
    ($ty:ident <$li:lifetime>) => {
        #[cfg(feature = "xlsx")]
        impl<$li> $ty<$li> {
            pub fn write_headers(
                worksheet: &mut rust_xlsxwriter::Worksheet,
                row: rust_xlsxwriter::RowNum,
                mut col: rust_xlsxwriter::ColNum,
                headers: &Vec<String>,
            ) -> Result<(), rust_xlsxwriter::XlsxError> {
                for header in headers.iter() {
                    worksheet.write(row, col, header)?;
                    col += 1;
                }
                Ok(())
            }

            pub fn write_data(
                worksheet: &mut rust_xlsxwriter::Worksheet,
                data: &[u8],
            ) -> Result<(), ImportError> {
                let mut reader = csv::Reader::from_reader(data);
                let mut record = csv::StringRecord::new();
                let headers = reader.headers()?.clone();
                while reader.read_record(&mut record)? {
                    println!("{:?}", record);
                    // let data = record.deserialize::<$ty>(Some(&headers))?;
                    // worksheet.serialize(&data)?;
                }
                Ok(())
            }
        }
    };
}

#[cfg(feature = "xlsx")]
#[cfg_attr(feature = "xlsx", derive(thiserror::Error, Debug))]
pub enum ImportError {
    #[error("Read error {0}")]
    Read(#[from] csv::Error),
    #[error("Write error {0}")]
    Write(#[from] rust_xlsxwriter::XlsxError),
}

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

// https://github.com/eprbell/rp2/blob/main/docs/input_files.md#in-transaction-table-format
#[derive(Debug)]
pub struct InputHeader(pub Vec<String>);

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

impl ToString for Input {
    fn to_string(&self) -> String {
        match self {
            Self::Airdrop => "airdrop".to_string(),
            Self::Buy => "buy".to_string(),
            Self::Donate => "donate".to_string(),
            Self::Gift => "gift".to_string(),
            Self::Hardfork => "hardfork".to_string(),
            Self::Income => "income".to_string(),
            Self::Interest => "interest".to_string(),
            Self::Mining => "mining".to_string(),
            Self::Staking => "staking".to_string(),
            Self::Wages => "wages".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct InputData<'a> {
    #[serde(with = "crate::date")]
    pub timestamp: DateTime<Utc>,
    pub asset: &'a str,
    pub exchange: &'a str,
    pub holder: &'a str,
    #[serde(rename(serialize = "transaction_type", deserialize = "type"))]
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

impl<'a> InputData<'a> {
    pub fn sort(headers: &Vec<String>, record: csv::StringRecord) -> csv::StringRecord {
        let mut sorted = csv::StringRecord::new();
        for (_idx, header) in headers.iter().enumerate() {
            match header.as_str() {
                "timestamp" => sorted.push_field(&record[0]),
                "asset" => sorted.push_field(&record[1]),
                "exchange" => sorted.push_field(&record[2]),
                "holder" => sorted.push_field(&record[3]),
                "transaction_type" => sorted.push_field(&record[4]),
                "type" => sorted.push_field(&record[4]),
                "spot_price" => sorted.push_field(&record[5]),
                "crypto_in" => sorted.push_field(&record[6]),
                "crypto_fee" => sorted.push_field(&record[7]),
                "fiat_in_no_fee" => sorted.push_field(&record[8]),
                "fiat_in_with_fee" => sorted.push_field(&record[9]),
                "fiat_fee" => sorted.push_field(&record[10]),
                "unique_id" => sorted.push_field(&record[11]),
                "notes" => sorted.push_field(&record[12]),
                field => {
                    warn!(field, "unknown header");
                    sorted.push_field("");
                }
            }
        }
        sorted
    }
}

impl_deserialize_header!(InputHeader);
impl_xlsx_writer!(InputData<'a>);

// https://github.com/eprbell/rp2/blob/main/docs/input_files.md#out-transaction-table-format
#[derive(Debug)]
pub struct OutputHeader(pub Vec<String>);

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
    #[serde(rename(serialize = "transaction_type", deserialize = "type"))]
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

impl_deserialize_header!(OutputHeader);
impl_xlsx_writer!(OutputData<'a>);

// https://github.com/eprbell/rp2/blob/main/docs/input_files.md#intra-transaction-table-format
#[derive(Debug)]
pub struct IntraHeader(pub Vec<String>);

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

impl_deserialize_header!(IntraHeader);
impl_xlsx_writer!(IntraData<'a>);

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

pub struct AssetTables<W>
where
    W: io::Write,
{
    pub input: csv::Writer<W>,
    pub output: csv::Writer<W>,
    pub intra: csv::Writer<W>,
}

impl<W> AssetTables<W>
where
    W: io::Write,
{
    pub fn into_inner(self) -> Result<(W, W, W), csv::IntoInnerError<csv::Writer<W>>> {
        Ok((
            self.input.into_inner()?,
            self.output.into_inner()?,
            self.intra.into_inner()?,
        ))
    }
}

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
        assert_eq!(expect_input, config.in_header.0.as_slice());
        assert_eq!(
            Some(&AccountingMethod::Hifo),
            config.accounting_methods.unwrap().year.get(&2022)
        );
    }
}
