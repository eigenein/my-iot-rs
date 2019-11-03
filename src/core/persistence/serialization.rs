//! `Value` serialization & deserialization for persistence.

use crate::prelude::*;
use std::convert::TryInto;

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

trait Serialize {
    fn serialize(&self) -> Vec<u8>;
}

impl Serialize for String {
    fn serialize(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

impl Serialize for PointOfTheCompass {
    fn serialize(&self) -> Vec<u8> {
        match self {
            PointOfTheCompass::North => 0u16,
            PointOfTheCompass::NorthNortheast => 1u16,
            PointOfTheCompass::Northeast => 2u16,
            PointOfTheCompass::EastNortheast => 3u16,
            PointOfTheCompass::East => 4u16,
            PointOfTheCompass::EastSoutheast => 5u16,
            PointOfTheCompass::Southeast => 6u16,
            PointOfTheCompass::SouthSoutheast => 7u16,
            PointOfTheCompass::South => 8u16,
            PointOfTheCompass::SouthSouthwest => 9u16,
            PointOfTheCompass::Southwest => 10u16,
            PointOfTheCompass::WestSouthwest => 11u16,
            PointOfTheCompass::West => 12u16,
            PointOfTheCompass::WestNorthwest => 13u16,
            PointOfTheCompass::Northwest => 14u16,
            PointOfTheCompass::NorthNorthwest => 15u16,
        }
        .serialize()
    }
}

impl Serialize for u8 {
    fn serialize(&self) -> Vec<u8> {
        vec![*self]
    }
}

impl Serialize for u16 {
    fn serialize(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl Serialize for u64 {
    fn serialize(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl Serialize for f64 {
    fn serialize(&self) -> Vec<u8> {
        self.to_bits().serialize()
    }
}

trait Deserialize
where
    Self: std::marker::Sized,
{
    fn deserialize(blob: &[u8]) -> Result<Self>;
}

impl Deserialize for String {
    fn deserialize(blob: &[u8]) -> Result<Self> {
        Ok(std::str::from_utf8(blob)?.into())
    }
}

impl Deserialize for u64 {
    fn deserialize(blob: &[u8]) -> Result<Self> {
        Ok(u64::from_le_bytes(blob.try_into()?))
    }
}

impl Deserialize for f64 {
    fn deserialize(blob: &[u8]) -> Result<Self> {
        Ok(f64::from_bits(u64::deserialize(blob)?))
    }
}

impl Deserialize for u16 {
    fn deserialize(blob: &[u8]) -> Result<Self> {
        Ok(u16::from_le_bytes(blob.try_into()?))
    }
}

impl Deserialize for PointOfTheCompass {
    fn deserialize(blob: &[u8]) -> Result<Self> {
        match u16::deserialize(&blob[0..2])? {
            0 => Ok(PointOfTheCompass::North),
            1 => Ok(PointOfTheCompass::NorthNortheast),
            2 => Ok(PointOfTheCompass::Northeast),
            3 => Ok(PointOfTheCompass::EastNortheast),
            4 => Ok(PointOfTheCompass::East),
            5 => Ok(PointOfTheCompass::EastSoutheast),
            6 => Ok(PointOfTheCompass::Southeast),
            7 => Ok(PointOfTheCompass::SouthSoutheast),
            8 => Ok(PointOfTheCompass::South),
            9 => Ok(PointOfTheCompass::SouthSouthwest),
            10 => Ok(PointOfTheCompass::Southwest),
            11 => Ok(PointOfTheCompass::WestSouthwest),
            12 => Ok(PointOfTheCompass::West),
            13 => Ok(PointOfTheCompass::WestNorthwest),
            14 => Ok(PointOfTheCompass::Northwest),
            15 => Ok(PointOfTheCompass::NorthNorthwest),
            value => Err(format_err!("invalid point of the compass value: {}", value)),
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
    value_ok!(
        wind_direction_ok,
        Value::WindDirection(PointOfTheCompass::NorthNortheast)
    );
    value_ok!(size_ok, Value::Size(42));
}
