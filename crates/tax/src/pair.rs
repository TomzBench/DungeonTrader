/// Pair
///
use std::borrow::Cow;

/// Helper for managing trade pairs (IE: BTC/USD, BTC/ETH)
use serde::{
    de::{self, Unexpected, Visitor},
    ser::{Serialize, Serializer},
    Deserialize,
};

#[derive(Debug, PartialEq)]
pub struct Pair<'a>(pub Cow<'a, str>, pub Cow<'a, str>);
impl<'a> Pair<'a> {
    pub fn new<I: Into<Cow<'a, str>>>(pair0: I, pair1: I) -> Self {
        Self(pair0.into(), pair1.into())
    }
}
impl<'a> Serialize for Pair<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}/{}", &self.0, &self.1))
    }
}

pub struct PairVisitor;
impl<'de> Visitor<'de> for PairVisitor {
    type Value = Pair<'de>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a currency pair string, ie: (BTC/ETH)")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if let Some((pair0, pair1)) = v.split_once('/') {
            Ok(Pair(Cow::Owned(pair0.into()), Cow::Owned(pair1.into())))
        } else {
            Err(de::Error::invalid_value(Unexpected::Str(v), &self))
        }
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Some((pair0, pair1)) = v.split_once('/') {
            Ok(Pair(Cow::Borrowed(pair0), Cow::Borrowed(pair1)))
        } else {
            Err(de::Error::invalid_value(Unexpected::Str(v), &self))
        }
    }
}

impl<'de: 'a, 'a> Deserialize<'de> for Pair<'a> {
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
        let pair = Pair::new("BTC", "USD");
        assert_tokens(&pair, &[Token::String("BTC/USD")]);
    }
}
