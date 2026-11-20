//! Field deserialization helpers.
//!
//! These functions are used through `#[serde(deserialize_with = "...")]` on event and
//! response struct fields to convert upstream values such as `null`, or numbers represented
//! as strings, into the field's Rust type.

use std::num::ParseIntError;

use serde::{Deserialize, Deserializer};

/// Converts JSON `null` to `T::default()` and deserializes any other value as `T`.
///
/// This function handles a field value only. A missing field is handled by
/// `#[serde(default)]` on the field and never reaches this function; `null` and a missing
/// field are distinct inputs. Types or formats that `T` cannot deserialize return a Serde
/// error.
pub fn null_to_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Option::<T>::deserialize(deserializer).map(|opt| opt.unwrap_or_default())
}

/// Accepts either a JSON number or a decimal string and parses it into an integer type
/// through `T: FromStr`.
///
/// Some upstream fields may appear as either a number or a string. The number branch only
/// accepts values representable as `i64`; the string branch delegates to `T::FromStr`. Other
/// JSON types return a Serde error (`type error`), and a failed string parse returns
/// `parse error`.
///
/// # Panics
///
/// If the input is a JSON number that cannot be represented as `i64` (for example a
/// floating-point number or an out-of-range integer), the current implementation calls
/// `unwrap` on the result of `as_i64()` and panics. Callers should ensure the upstream sends
/// integer values only.
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
