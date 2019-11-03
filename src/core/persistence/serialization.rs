//! `Value` serialization & deserialization for persistence.

use crate::prelude::*;
use std::convert::TryInto;

impl Value {
    pub fn serialize(&self) -> (u32, Vec<u8>) {
        match self {
            Value::None => (0, vec![]),
            Value::Boolean(value) => (if !value { 1 } else { 2 }, vec![]),
            Value::ImageUrl(value) => (3, serialize_string(value)),
            Value::Text(value) => (4, serialize_string(value)),
            Value::Bft(value) => (5, vec![*value]),
            Value::Celsius(value) => (6, value.to_bits().to_le_bytes().to_vec()),
            Value::Counter(value) => (7, serialize_u64(*value)),
            Value::Metres(value) => (8, serialize_f64(*value)),
            Value::Rh(value) => (9, serialize_f64(*value)),
            Value::WindDirection(value) => (10, vec![*value as u8]),
            Value::Size(value) => (11, serialize_u64(*value)),
        }
    }

    pub fn deserialize(type_: u32, blob: Vec<u8>) -> Result<Self> {
        match type_ {
            0 => Ok(Value::None),
            1 => Ok(Value::Boolean(false)),
            2 => Ok(Value::Boolean(true)),
            3 => Ok(Value::ImageUrl(deserialize_string(blob)?)),
            4 => Ok(Value::Text(deserialize_string(blob)?)),
            5 => Ok(Value::Bft(blob[0])),
            6 => Ok(Value::Celsius(deserialize_f64(&blob[0..8])?)),
            7 => Ok(Value::Counter(deserialize_u64(&blob[0..8])?)),
            8 => Ok(Value::Metres(deserialize_f64(&blob[0..8])?)),
            // TODO: 8, 9, 10, 11
            _ => Err(format_err!("unknown value type: {}", type_)),
        }
    }
}

fn serialize_string(string: &str) -> Vec<u8> {
    string.as_bytes().to_vec()
}

fn serialize_u64(value: u64) -> Vec<u8> {
    value.to_le_bytes().to_vec()
}

fn serialize_f64(value: f64) -> Vec<u8> {
    serialize_u64(value.to_bits())
}

fn deserialize_string(blob: Vec<u8>) -> Result<String> {
    Ok(String::from_utf8(blob)?)
}

fn deserialize_u64(blob: &[u8]) -> Result<u64> {
    Ok(u64::from_le_bytes(blob.try_into()?))
}

fn deserialize_f64(blob: &[u8]) -> Result<f64> {
    Ok(f64::from_bits(deserialize_u64(blob)?))
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
    value_ok!(
        wind_direction_ok,
        Value::WindDirection(PointOfTheCompass::NorthNortheast)
    );
    value_ok!(size_ok, Value::Size(42));
}
