/// Helper for managing trade pairs (IE: BTC/USD, BTC/ETH)
use serde::{
    de::{self, Unexpected, Visitor},
    ser::{Serialize, Serializer},
    Deserialize,
};

#[derive(Debug, PartialEq)]
pub struct Pair(String, String);
impl Serialize for Pair {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}/{}", &self.0, &self.1))
    }
}

pub struct PairVisitor;
impl<'de> Visitor<'de> for PairVisitor {
    type Value = Pair;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a currency pair string, ie: (BTC/ETH)")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Some((pair0, pair1)) = v.split_once('/') {
            Ok(Pair(pair0.to_owned(), pair1.to_owned()))
        } else {
            Err(de::Error::invalid_value(Unexpected::Str(v), &self))
        }
    }
}

impl<'de> Deserialize<'de> for Pair {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(PairVisitor)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_test::{assert_tokens, Token};

    #[test]
    fn should_serialize() {
        let pair = Pair("BTC".to_string(), "USD".to_string());
        assert_tokens(&pair, &[Token::String("BTC/USD")]);
    }
}
