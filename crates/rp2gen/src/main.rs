/// rp2gen
use anyhow::Result;
use clap::{
    arg,
    builder::{EnumValueParser, PossibleValue},
    command, value_parser, Arg, ArgAction, ValueEnum,
};
use dungeon_tax::sheet::{AssetTables, Config, InputData, IntraData, OutputData};
use rust_xlsxwriter::Workbook;
use std::{collections::HashMap, fs, iter::zip, path::PathBuf};
use tracing::info;
use tracing_subscriber::{filter::LevelFilter, fmt, layer::SubscriberExt, prelude::*};

fn main() -> Result<()> {
    // Parse command line
    let matches = command!()
        .arg(
            arg!(-L --"log-level" [Level] "trace, debug, info, warn, error")
                .value_parser(EnumValueParser::<LogLevel>::new())
                .required(false),
        )
        .arg(
            arg!(-C --config <FILE> "Path to INI config file")
                .value_parser(value_parser!(PathBuf))
                .action(ArgAction::Set)
                .required(true),
        )
        .arg(
            arg!(-i --input <FILE> "CSV input (stdout if empty)")
                .value_parser(value_parser!(PathBuf))
                .action(ArgAction::Set)
                .required(true),
        )
        .arg(
            Arg::new("output")
                .value_parser(value_parser!(PathBuf))
                .action(ArgAction::Set)
                .default_value("output.ods"),
        )
        .get_matches();

    // Setup logging
    if let Some(level) = matches.get_one::<LogLevel>("log-level") {
        let stdout = fmt::layer()
            .compact()
            .with_ansi(true)
            .with_level(true)
            .with_file(false)
            .with_line_number(false)
            .with_target(true);
        tracing_subscriber::registry()
            .with(stdout)
            .with(LevelFilter::from(*level))
            .init();
    }

    // Log hello
    info!("starting import");

    // Parse the config
    let path_config = matches
        .get_one::<std::path::PathBuf>("config")
        .expect("`config` missing arg");
    let config: Config = dungeon_ini::from_str(&fs::read_to_string(path_config)?)?;

    // Get a array of crypto currency ticker symbols for which we are interested in
    let assets: Vec<&'_ str> = config.general.assets.iter().map(|a| a.as_str()).collect();

    // Get some buffers to write csv data to
    let buffers: Vec<_> = config
        .general
        .assets
        .iter()
        .map(|_asset| AssetTables {
            input: csv::Writer::from_writer(Vec::new()),
            output: csv::Writer::from_writer(Vec::new()),
            intra: csv::Writer::from_writer(Vec::new()),
        })
        .collect();

    // Parse the exchange input csv data (TODO parse multiple exchanges)
    let path_input = matches
        .get_one::<PathBuf>("input")
        .expect("`input` missing arg");
    let mut input = fs::File::open(path_input)?;

    // Import our csv data into the asset tables
    let mut asset_tables: HashMap<_, _> = zip(assets, buffers).into_iter().collect();
    dungeon_kraken::import::from_reader(&config, &mut input, &mut asset_tables)?;

    // Write csv data into workbook
    let mut workbook = Workbook::new();
    for (asset, tables) in asset_tables.drain() {
        // Get the buffers and create a worksheet
        let (input, output, intra) = tables.into_inner()?;
        let worksheet = workbook.add_worksheet().set_name(asset)?;

        // Write the input table
        worksheet.write(0, 0, "IN")?;
        InputData::write_headers(worksheet, 1, 0, &config.in_header.0)?;
        InputData::write_data(worksheet, input.as_slice())?;
        // let (_, _, row, _) = worksheet.get_serialize_dimensions("InputData")?;
        worksheet.write(2, 0, "TABLE END")?;

        // Write the output table
        // worksheet.write(row + 3, 0, "OUT")?;
        // OutputData::write_headers(worksheet, row + 4, 0, &config.out_header.0)?;
        // OutputData::write_data(worksheet, output.as_slice())?;
        // let (_, _, row, _) = worksheet.get_serialize_dimensions("OutputData")?;
        // worksheet.write(row + 1, 0, "TABLE END")?;

        // // Write the intra table
        // worksheet.write(row + 3, 0, "INTRA")?;
        // IntraData::write_headers(worksheet, row + 4, 0, &config.intra_header.0)?;
        // IntraData::write_data(worksheet, intra.as_slice())?;
        // let (_, _, row, _) = worksheet.get_serialize_dimensions("IntraData")?;
        // worksheet.write(row + 1, 0, "TABLE END")?;
    }

    let output = matches
        .get_one::<PathBuf>("output")
        .expect("missing output arg");
    workbook.save(output)?;

    Ok(())
}

#[derive(Debug, Copy, Clone)]
enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}
impl ValueEnum for LogLevel {
    fn from_str(input: &str, _ignore_case: bool) -> std::result::Result<Self, String> {
        match input {
            "trace" => Ok(LogLevel::Trace),
            "debug" => Ok(LogLevel::Trace),
            "info" => Ok(LogLevel::Trace),
            "warn" => Ok(LogLevel::Trace),
            "error" => Ok(LogLevel::Trace),
            _ => Err("invalid log level, valid levels".to_string()),
        }
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self {
            LogLevel::Trace => PossibleValue::new("trace").help("display every possible log level"),
            LogLevel::Debug => PossibleValue::new("debug").help("diagnostic logs only"),
            LogLevel::Info => PossibleValue::new("info").help("quiet logging"),
            LogLevel::Warn => PossibleValue::new("warn").help("only log warning and above"),
            LogLevel::Error => PossibleValue::new("error").help("critical errors logged only"),
        })
    }

    fn value_variants<'a>() -> &'a [Self] {
        &[
            LogLevel::Trace,
            LogLevel::Debug,
            LogLevel::Info,
            LogLevel::Warn,
            LogLevel::Error,
        ]
    }
}

impl From<LogLevel> for LevelFilter {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Trace => LevelFilter::TRACE,
            LogLevel::Debug => LevelFilter::DEBUG,
            LogLevel::Info => LevelFilter::INFO,
            LogLevel::Warn => LevelFilter::WARN,
            LogLevel::Error => LevelFilter::ERROR,
        }
    }
}
