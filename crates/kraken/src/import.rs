use crate::account::{BuySell, TradesExport};
use dungeon_tax::sheet::{self, AssetMap};
use std::{fs, io, path};
use tracing::{trace, warn};

#[derive(thiserror::Error, Debug)]
pub enum ImportError {
    #[error("Invalid data => {0}")]
    Csv(#[from] csv::Error),
    #[error("Unknown Asset {0}")]
    UnknownAsset(String),
}

pub fn from_reader<R, W>(
    config: &sheet::Config,
    src: R,
    dst: &mut AssetMap<'_, W>,
) -> Result<(), ImportError>
where
    R: io::Read,
    W: io::Write,
{
    let mut reader = csv::Reader::from_reader(src);
    let headers = reader.headers()?.clone();
    trace!(headers=?headers, "importing file");
    let mut record = csv::StringRecord::new();
    while reader.read_record(&mut record)? {
        let trade = record.deserialize::<TradesExport>(Some(&headers))?;
        let base_currency = trade.pair.0.as_ref();
        let quote_currency = trade.pair.1.as_ref();
        if config
            .general
            .assets
            .iter()
            .any(|s| *s == trade.pair.1.as_ref())
        {
            let (o, i) = convert(config, &trade);
            dst.get_mut(quote_currency)
                .ok_or_else(|| ImportError::UnknownAsset(quote_currency.to_string()))?
                .1
                .serialize(&o)?;
            dst.get_mut(base_currency)
                .ok_or_else(|| ImportError::UnknownAsset(base_currency.to_string()))?
                .0
                .serialize(&i)?;
            trace!(
                date = o.timestamp.to_string(),
                base_currency,
                quote_currency,
                usd = trade.costusd,
                "convert"
            );
        } else if trade.typ == BuySell::Buy {
            let tx = buy(config, &trade);
            dst.get_mut(base_currency)
                .ok_or_else(|| ImportError::UnknownAsset(base_currency.to_string()))?
                .0
                .serialize(&tx)?;
            trace!(
                date = tx.timestamp.to_string(),
                base_currency,
                usd = trade.costusd,
                "buy"
            );
        } else if trade.typ == BuySell::Sell {
            let tx = sell(config, &trade);
            dst.get_mut(base_currency)
                .ok_or_else(|| ImportError::UnknownAsset(base_currency.to_string()))?
                .1
                .serialize(&tx)?;
            trace!(
                date = tx.timestamp.to_string(),
                base_currency,
                usd = trade.costusd,
                "sell"
            );
        }
    }
    Ok(())
}

fn buy<'a, 'de>(config: &'a sheet::Config, trade: &'a TradesExport<'de>) -> sheet::InputData<'a> {
    let fiat_fee = trade.fee * trade.costusd / trade.cost;
    let holder = config
        .general
        .holders
        .get(0)
        .map(|s| s.as_ref())
        .unwrap_or("_UNKNOWN");
    sheet::InputData {
        timestamp: trade.time,
        asset: &trade.pair.0,
        exchange: "kraken",
        holder,
        typ: sheet::Input::Buy,
        spot_price: trade.price,
        crypto_in: trade.vol,
        crypto_fee: Some(0.0),
        fiat_in_no_fee: Some(trade.costusd), // vol * price
        fiat_in_with_fee: Some(trade.costusd + fiat_fee),
        fiat_fee,
        unique_id: trade.txid,
        notes: Some(trade.ordertxid),
    }
}

fn sell<'a, 'de>(config: &'a sheet::Config, trade: &'a TradesExport<'de>) -> sheet::OutputData<'a>
where
    'de: 'a,
{
    let fiat_fee = trade.fee * trade.costusd / trade.cost;
    let holder = config
        .general
        .holders
        .get(0)
        .map(|s| s.as_ref())
        .unwrap_or("_UNKNOWN");
    sheet::OutputData {
        timestamp: trade.time,
        asset: &trade.pair.0,
        exchange: "kraken",
        holder,
        typ: sheet::Output::Sell,
        spot_price: trade.price,
        crypto_out_no_fee: trade.vol,
        crypto_out_with_fee: Some(trade.vol),
        crypto_fee: 0.0,
        fiat_out_no_fee: Some(trade.costusd),
        fiat_fee: Some(fiat_fee),
        unique_id: trade.txid,
        notes: Some(trade.ordertxid),
    }
}

fn convert<'a, 'de>(
    config: &'a sheet::Config,
    trade: &'a TradesExport<'de>,
) -> (sheet::OutputData<'a>, sheet::InputData<'a>) {
    let holder = config
        .general
        .holders
        .get(0)
        .map(|s| s.as_str())
        .unwrap_or(&"_UNKNOWN");
    let sell = sheet::OutputData {
        timestamp: trade.time,
        asset: &trade.pair.1,
        exchange: "kraken",
        holder,
        typ: sheet::Output::Sell,
        spot_price: trade.costusd / trade.cost,
        crypto_out_no_fee: trade.cost,
        crypto_out_with_fee: Some(trade.cost + trade.fee),
        crypto_fee: trade.fee,
        fiat_out_no_fee: Some(trade.costusd),
        fiat_fee: Some(trade.fee * trade.costusd / trade.cost),
        unique_id: trade.txid,
        notes: Some(trade.ordertxid),
    };
    let buy = sheet::InputData {
        timestamp: trade.time,
        asset: &trade.pair.0,
        exchange: "kraken",
        holder,
        typ: sheet::Input::Buy,
        spot_price: (trade.price * trade.costusd) / trade.cost,
        crypto_in: trade.vol,
        crypto_fee: Some(0.0), // Captured on output side of transaction
        fiat_in_no_fee: Some(trade.costusd),
        fiat_in_with_fee: Some(trade.costusd),
        fiat_fee: 0.0,
        unique_id: trade.txid,
        notes: Some(trade.ordertxid),
    };
    (sell, buy)
}
