//! `Value` serialization & deserialization for persistence.

use crate::core::persistence::primitives::{Deserialize, Serialize};
use crate::prelude::*;

impl Value {
    pub fn serialize(&self) -> (u32, Vec<u8>) {
        match self {
            Value::None => (0, vec![]),
            Value::Boolean(value) => (if !value { 1 } else { 2 }, vec![]),
            Value::ImageUrl(value) => (3, value.serialize()),
            Value::Text(value) => (4, value.serialize()),
            Value::Bft(value) => (5, value.serialize()),
            Value::Celsius(value) => (6, value.serialize()),
            Value::Counter(value) => (7, value.serialize()),
            Value::Metres(value) => (8, value.serialize()),
            Value::Rh(value) => (9, value.serialize()),
            Value::WindDirection(value) => (10, value.serialize()),
            Value::Size(value) => (11, value.serialize()),
        }
    }

    pub fn deserialize(type_: u32, blob: Vec<u8>) -> Result<Self> {
        match type_ {
            0 => Ok(Value::None),
            1 => Ok(Value::Boolean(false)),
            2 => Ok(Value::Boolean(true)),
            3 => Ok(Value::ImageUrl(String::deserialize(&blob)?)),
            4 => Ok(Value::Text(String::deserialize(&blob)?)),
            5 => Ok(Value::Bft(blob[0])),
            6 => Ok(Value::Celsius(f64::deserialize(&blob[0..8])?)),
            7 => Ok(Value::Counter(u64::deserialize(&blob[0..8])?)),
            8 => Ok(Value::Metres(f64::deserialize(&blob[0..8])?)),
            9 => Ok(Value::Rh(f64::deserialize(&blob[0..8])?)),
            10 => Ok(Value::WindDirection(PointOfTheCompass::deserialize(&blob[0..2])?)),
            11 => Ok(Value::Size(u64::deserialize(&blob[0..8])?)),
            _ => Err(format_err!("unknown value type: {}", type_)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! value_ok {
        ($name:ident, $value:expr) => {
            #[test]
            fn $name() -> crate::Result<()> {
                let value = $value;
                let (type_, blob) = value.serialize();
                assert_eq!(Value::deserialize(type_, blob)?, value);
                Ok(())
            }
        };
    }

    value_ok!(none_ok, Value::None);
    value_ok!(false_ok, Value::Boolean(false));
    value_ok!(true_ok, Value::Boolean(true));
    value_ok!(image_url_ok, Value::ImageUrl("https://google.com".into()));
    value_ok!(text_ok, Value::Text("Hello, world!".into()));
    value_ok!(bft_ok, Value::Bft(3));
    value_ok!(celsius_ok, Value::Celsius(42.0));
    value_ok!(counter_ok, Value::Counter(42));
    value_ok!(metres_ok, Value::Metres(42.0));
    value_ok!(rh_ok, Value::Rh(42.0));
    value_ok!(wind_direction_ok, Value::WindDirection(PointOfTheCompass::South));
    value_ok!(size_ok, Value::Size(42));
}
