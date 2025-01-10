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

    // Parse input
    let csv = csv::Reader::from_reader(std::io::stdin())
        .deserialize::<kraken::account::TradesExport>()
        .collect::<csv::Result<Vec<kraken::account::TradesExport>>>()?;

    // convert into JSON
    let json = serde_json::to_string_pretty(&csv)?;

    // Write output
    write!(std::io::stdout(), "{}", &json)?;

    // Done
    Ok(())
}
