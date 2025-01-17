/// date
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Deserializer, Serializer};
const FORMAT: &str = "%Y-%m-%d %H:%M:%S.%f";
const FORMAT_TZ: &str = "%Y-%m-%d %H:%M:%S.%f%:z";

pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("{}", date.format(FORMAT_TZ)))
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let dt = NaiveDateTime::parse_from_str(&s, FORMAT_TZ);
    match dt {
        Ok(dt) => Ok(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc)),
        Err(_) => NaiveDateTime::parse_from_str(&s, FORMAT)
            .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
            .map_err(serde::de::Error::custom),
    }
}
