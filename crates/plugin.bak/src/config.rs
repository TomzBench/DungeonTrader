/// config.rs
///
/// There serde INI is not maintained, so we use NOM to build an intermidiate representation of the
/// serde data model and then make our own ini deserializer
///
/// See more about the configuration here:
/// https://github.com/eprbell/rp2/blob/main/docs/input_files.md#the-config-file
use nom::{
    branch::alt,
    bytes::complete::{take_till, take_until1, take_while, take_while1},
    character::{
        complete::{alpha1, alphanumeric1, char, digit1, line_ending, multispace0, space0},
        is_alphabetic, is_alphanumeric, is_space,
    },
    combinator::{cond, eof, map, map_res, opt, peek, recognize},
    error::ParseError,
    multi::{many0, many1, separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, terminated, tuple, Tuple},
    IResult, Parser,
};
use serde::Deserialize;
use std::{collections::HashMap, iter::FromIterator, str};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct General {
    pub assets: Vec<String>,
    pub exchanges: Vec<String>,
    pub holder: String,
    pub spouse: Option<String>,
    pub generator: Option<String>,
    pub extra: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct InHeader {
    pub timestamp: u8,
    pub asset: u8,
    pub exchange: u8,
    pub holder: u8,
    pub transaction_type: u8,
    pub spot_price: u8,
    pub crypto_in: u8,
    pub crypto_fee: u8,
    pub fiat_fee: u8,
    pub fiat_in_no_fee: Option<u8>,
    pub fiat_in_with_fee: Option<u8>,
    pub notes: Option<u8>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OutHeader {
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountingMethod {
    Fifo,
    Lifo,
    Hifo,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AccountingMethods {
    pub year: HashMap<u16, AccountingMethod>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub general: General,
    pub in_header: InHeader,
    pub out_header: OutHeader,
    pub intra_header: IntraHeader,
    pub accounting_methods: Option<AccountingMethods>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value<'a> {
    Num(i64),
    Str(&'a str),
    Array(Vec<Value<'a>>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Key<'a> {
    Num(i64),
    Str(&'a str),
}

impl From<i64> for Key<'_> {
    fn from(value: i64) -> Self {
        Key::Num(value)
    }
}

impl<'a> From<&'a str> for Key<'a> {
    fn from(value: &'a str) -> Self {
        Key::Str(value)
    }
}

// Skip everything that isnt parsed already upto a line_ending.
// TODO we should replace opt(line_ending) with alt(line_ending,eof)
pub fn line<'a, O, E: ParseError<&'a str>, F>(
    mut first: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Parser<&'a str, O, E>,
{
    move |i: &'a str| {
        let (input, o) = first.parse(i)?;
        let (rest, _) = terminated(take_till(|c| c == '\n'), opt(line_ending))(input)?;
        Ok((rest, o))
    }
}

fn is_alphanumeric_with_spaces(c: char) -> bool {
    is_alphanumeric(c as u8) || is_space(c as u8)
}

// Skip leading whitespace, and skip trailing comments.
fn one_value(i: &str) -> IResult<&str, Value> {
    preceded(
        multispace0,
        alt((
            map_res(digit1, |s| str::parse::<i64>(s).map(Value::Num)),
            map(take_while(is_alphanumeric_with_spaces), Value::Str),
        )),
    )(i)
}

fn value(i: &str) -> IResult<&str, Value> {
    match peek(terminated(one_value, char(',')))(i).is_ok() {
        true => map(separated_list1(char(','), one_value), Value::Array)(i),
        false => one_value(i),
    }
}

fn key_value(i: &str) -> IResult<&str, (Key, Value)> {
    line(pair(
        map(alphanumeric1, Key::Str),
        preceded(tuple((opt(space0), char('='), opt(space0))), value),
    ))(i)
}

fn category(i: &str) -> IResult<&str, (&str, Option<&str>)> {
    line(delimited(
        char('['),
        pair(alpha1, opt(preceded(char(' '), alpha1))),
        char(']'),
    ))(i)
}

fn group(i: &str) -> IResult<&str, (&str, HashMap<Key, Value>)> {
    pair(
        map(category, |(cat, _meta)| cat),
        map(many0(key_value), |v| v.into_iter().collect()),
    )(i)
}

#[cfg(test)]
mod tests {

    use super::{category, group, key_value, Key, Value};
    use indoc::indoc;

    #[test]
    fn should_parse_empty_group() {
        let input = "[general]";
        let (_, (table, map)) = group(input).unwrap();
        assert_eq!("general", table);
        assert_eq!(0, map.len());
    }

    #[test]
    fn should_parse_group() {
        let input = indoc! {r#"
            [general]
            foo = bar
            num = 42
        "#};
        let (_, (table, map)) = group(input).unwrap();
        assert_eq!("[general]\nfoo = bar\nnum = 42\n", input);
        println!("{:?}", map);
        assert_eq!("general", table);
        assert_eq!(2, map.len());
        assert_eq!(Some(&Value::Str("bar")), map.get(&"foo".into()));
        assert_eq!(Some(&Value::Num(42)), map.get(&"num".into()));
    }

    #[test]
    fn should_parse_category() {
        let parse = category("[something]");
        assert_eq!(Result::Ok(("", ("something", None))), parse);

        let parse = category("[something foo]");
        assert_eq!(Result::Ok(("", ("something", Some("foo")))), parse);

        let parse = category("[something foo] haha");
        assert_eq!(Result::Ok(("", ("something", Some("foo")))), parse);

        let parse = category("[something foo]\n");
        assert_eq!(Result::Ok(("", ("something", Some("foo")))), parse);

        let parse = category("[something foo] ; haha");
        assert_eq!(Result::Ok(("", ("something", Some("foo")))), parse);
    }

    #[test]
    fn should_parse_key_value() {
        let parse = key_value("hello=world");
        assert_eq!(
            Result::Ok(("", (Key::Str("hello"), Value::Str("world")))),
            parse
        );

        let parse = key_value("hello=world\n");
        assert_eq!(
            Result::Ok(("", (Key::Str("hello"), Value::Str("world")))),
            parse
        );

        let parse = key_value("hello=world \n");
        assert_eq!(
            Result::Ok(("", (Key::Str("hello"), Value::Str("world ")))),
            parse
        );

        let parse = key_value("hello=world\nnext");
        assert_eq!(
            Result::Ok(("next", (Key::Str("hello"), Value::Str("world")))),
            parse
        );

        let parse = key_value("hello=world ;foo\nnext");
        assert_eq!(
            Result::Ok(("next", (Key::Str("hello"), Value::Str("world ")))),
            parse
        );

        let parse = key_value("hello=world;foo\nnext");
        assert_eq!(
            Result::Ok(("next", (Key::Str("hello"), Value::Str("world")))),
            parse
        );

        // word with spaces
        let parse = key_value("hello = tom foo");
        assert_eq!(
            Result::Ok(("", (Key::Str("hello"), Value::Str("tom foo")))),
            parse
        );

        // number
        let parse = key_value("hello=42");
        assert_eq!(Result::Ok(("", (Key::Str("hello"), Value::Num(42)))), parse);

        // number weird spacing
        let parse = key_value("hello=   42  ; foop ");
        assert_eq!(Result::Ok(("", (Key::Str("hello"), Value::Num(42)))), parse);

        // spaces around '=', mixed vectors, has remaining
        let parse = key_value("hello = one, 2, three;//  one, two\nnext");
        assert_eq!(
            Result::Ok((
                "next",
                (
                    Key::Str("hello"),
                    Value::Array(vec![Value::Str("one"), Value::Num(2), Value::Str("three")])
                )
            )),
            parse
        );
    }

    #[test]
    fn should_parse() {
        // TODO use INI support
        let input = indoc! {r#"
            [general sfdsdfs]
            assets = B1, B2, B3, B4
            exchanges = BlockFi, Coinbase, Coinbase Pro, Kraken
            holders = Bob, Alice
            
            [accounting_methods]
            2020 = fifo
            2021 = lifo
            2022 = hifo
            2023 = fifo
            
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
            notes = 12
            
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
        "#};
        //let _config: Config = toml::from_str(input).unwrap();
    }
}
