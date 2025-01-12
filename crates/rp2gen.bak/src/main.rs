use anyhow::Result;
/// rp2gen
use clap::{arg, command};
use dt_plugin_kraken as kraken;
use std::io::Write;

fn main() -> Result<()> {
    // Parse command line
    let matches = command!()
        .arg(arg!(--"kraken-api-key" <KEY> "Kraken api key").env("KRAKEN_API_KEY"))
        .arg(arg!(--"kraken-private-key" <KEY> "Kraken private key").env("KRAKEN_PRIVATE_KEY"))
        .arg(arg!(-i --input <FILE> "CSV input (stdin if empty)"))
        .arg(arg!(-o --output <FILE> "CSV output (stdout if empty)"))
        .get_matches();

    let mut rdr = csv::Reader::from_reader(std::io::stdin());
    let mut records = csv::StringRecord::new();
    let headers = rdr.headers()?.clone();

    while rdr.read_record(&mut records)? {
        let record: kraken::account::TradesExport = records.deserialize(Some(&headers))?;
        let json = serde_json::to_string_pretty(&record)?;
        writeln!(std::io::stdout(), "{}", &json)?;
    }

    Ok(())
}
