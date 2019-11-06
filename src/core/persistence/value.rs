//! `Value` serialization & deserialization for persistence.

use crate::core::persistence::primitives::{Deserialize, Serialize};
use crate::prelude::*;

impl Value {
    pub fn serialize(&self) -> Vec<u8> {
        match self {
            Value::None => vec![0],
            Value::Boolean(value) => {
                if !value {
                    vec![1]
                } else {
                    vec![2]
                }
            }
            Value::ImageUrl(value) => [vec![3], value.serialize()].concat(),
            Value::Text(value) => [vec![4], value.serialize()].concat(),
            Value::Bft(value) => [vec![5], value.serialize()].concat(),
            Value::Celsius(value) => [vec![6], value.serialize()].concat(),
            Value::Counter(value) => [vec![7], value.serialize()].concat(),
            Value::Metres(value) => [vec![8], value.serialize()].concat(),
            Value::Rh(value) => [vec![9], value.serialize()].concat(),
            Value::WindDirection(value) => [vec![10], value.serialize()].concat(),
            Value::Size(value) => [vec![11], value.serialize()].concat(),
        }
    }

    pub fn deserialize(blob: Vec<u8>) -> Result<Self> {
        match blob[0] {
            0 => Ok(Value::None),
            1 => Ok(Value::Boolean(false)),
            2 => Ok(Value::Boolean(true)),
            3 => Ok(Value::ImageUrl(String::deserialize(&blob[1..])?)),
            4 => Ok(Value::Text(String::deserialize(&blob[1..])?)),
            5 => Ok(Value::Bft(blob[1])),
            6 => Ok(Value::Celsius(f64::deserialize(&blob[1..9])?)),
            7 => Ok(Value::Counter(u64::deserialize(&blob[1..9])?)),
            8 => Ok(Value::Metres(f64::deserialize(&blob[1..9])?)),
            9 => Ok(Value::Rh(f64::deserialize(&blob[1..9])?)),
            10 => Ok(Value::WindDirection(PointOfTheCompass::deserialize(&blob[1..2])?)),
            11 => Ok(Value::Size(u64::deserialize(&blob[1..9])?)),
            type_ => Err(format_err!("unknown value type: {}", type_)),
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
                assert_eq!(Value::deserialize(value.serialize())?, value);
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
