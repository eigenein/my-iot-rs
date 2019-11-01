//! `Value` serialization & deserialization for persistence.

use crate::prelude::*;
use std::convert::TryInto;

impl Value {
    pub fn serialize(&self) -> (u32, Vec<u8>) {
        match self {
            Value::None => (0, vec![]),
            Value::Boolean(value) => (if !value { 1 } else { 2 }, vec![]),
            Value::ImageUrl(value) => (3, value.as_bytes().to_vec()),
            Value::Text(value) => (4, value.as_bytes().to_vec()),
            Value::Bft(value) => (5, vec![*value]),
            Value::Celsius(value) => (6, value.to_bits().to_le_bytes().to_vec()),
            Value::Counter(value) => (7, value.to_le_bytes().to_vec()),
            Value::Metres(value) => (8, value.to_bits().to_le_bytes().to_vec()),
            Value::Rh(value) => (9, value.to_bits().to_le_bytes().to_vec()),
            Value::WindDirection(value) => (10, (*value as u32).to_le_bytes().to_vec()),
            Value::Size(value) => (11, value.to_le_bytes().to_vec()),
        }
    }

    pub fn deserialize(type_: u32, blob: Vec<u8>) -> Result<Self> {
        match type_ {
            0 => Ok(Value::None),
            1 => Ok(Value::Boolean(false)),
            2 => Ok(Value::Boolean(true)),
            3 => Ok(Value::ImageUrl(String::from_utf8(blob)?)),
            4 => Ok(Value::Text(String::from_utf8(blob)?)),
            5 => Ok(Value::Bft(blob[0])),
            6 => Ok(Value::Celsius(f64::from_bits(u64::from_le_bytes(
                (&blob[0..8]).try_into()?,
            )))),
            7 => Ok(Value::Counter(u64::from_le_bytes((&blob[0..8]).try_into()?))),
            // TODO: 8, 9, 10, 11
            _ => Err(format_err!("unknown value type: {}", type_)),
        }
    }
}
