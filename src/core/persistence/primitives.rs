use crate::prelude::*;
use std::convert::TryInto;

pub trait Serialize {
    fn serialize(&self) -> Vec<u8>;
}

impl Serialize for String {
    fn serialize(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

impl Serialize for PointOfTheCompass {
    fn serialize(&self) -> Vec<u8> {
        vec![match self {
            PointOfTheCompass::North => 0,
            PointOfTheCompass::NorthNortheast => 1,
            PointOfTheCompass::Northeast => 2,
            PointOfTheCompass::EastNortheast => 3,
            PointOfTheCompass::East => 4,
            PointOfTheCompass::EastSoutheast => 5,
            PointOfTheCompass::Southeast => 6,
            PointOfTheCompass::SouthSoutheast => 7,
            PointOfTheCompass::South => 8,
            PointOfTheCompass::SouthSouthwest => 9,
            PointOfTheCompass::Southwest => 10,
            PointOfTheCompass::WestSouthwest => 11,
            PointOfTheCompass::West => 12,
            PointOfTheCompass::WestNorthwest => 13,
            PointOfTheCompass::Northwest => 14,
            PointOfTheCompass::NorthNorthwest => 15,
        }]
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

pub trait Deserialize
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
        match blob[0] {
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
            value => Err(InternalError::new(format!("invalid point of the compass value: {}", value)).into()),
        }
    }
}
