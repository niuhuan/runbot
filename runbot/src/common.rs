use std::num::ParseIntError;

use serde::{Deserialize, Deserializer};

pub fn null_to_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Option::<T>::deserialize(deserializer).map(|opt| opt.unwrap_or_default())
}

pub fn fuzzy_int<'de, D, T>(d: D) -> std::result::Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: std::str::FromStr<Err = ParseIntError>,
{
    let value: serde_json::Value = serde::Deserialize::deserialize(d)?;
    if value.is_number() {
        let number = value.as_i64().unwrap();
        let from: std::result::Result<T, ParseIntError> =
            std::str::FromStr::from_str(number.to_string().as_str());
        match from {
            Ok(from) => Ok(from),
            Err(_) => Err(serde::de::Error::custom("parse error")),
        }
    } else if value.is_string() {
        let str = value.as_str().unwrap();
        let from: std::result::Result<T, ParseIntError> = std::str::FromStr::from_str(str);
        match from {
            Ok(from) => Ok(from),
            Err(_) => Err(serde::de::Error::custom("parse error")),
        }
    } else {
        Err(serde::de::Error::custom("type error"))
    }
}
