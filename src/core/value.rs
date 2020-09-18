//! Implements sensor reading value.

use crate::prelude::*;
use serde::{de, Deserialize, Serialize, Serializer};

use bytes::Bytes;
use std::sync::Arc;

pub mod from;
pub mod try_into;

/// Sensor reading value.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Value {
    /// No value.
    None,

    /// Generic counter.
    Counter(i64),

    /// Image URL.
    ImageUrl(String),

    /// Boolean.
    Boolean(bool),

    /// Size in [bytes](https://en.wikipedia.org/wiki/Byte).
    DataSize(i64),

    /// [Plain text](https://en.wikipedia.org/wiki/Plain_text).
    Text(String),

    /// [Beaufort](https://en.wikipedia.org/wiki/Beaufort_scale) wind speed.
    Bft(u8),

    /// [Relative humidity](https://en.wikipedia.org/wiki/Relative_humidity) in **percents**.
    Rh(f64),

    /// [Thermodynamic temperature](https://en.wikipedia.org/wiki/Thermodynamic_temperature)
    /// in [Celsius](https://en.wikipedia.org/wiki/Celsius).
    Temperature(f64),

    /// [Length](https://en.wikipedia.org/wiki/Length) in meters.
    Length(f64),

    /// Duration in seconds.
    Duration(f64),

    /// Relative intensity in percents.
    RelativeIntensity(f64),

    /// [Power](https://en.wikipedia.org/wiki/Power_(physics))
    /// in [Watt](https://en.wikipedia.org/wiki/Watt)s.
    Power(f64),

    /// [Volume](https://en.wikipedia.org/wiki/Volume) in cubic meters.
    Volume(f64),

    /// [Energy](https://en.wikipedia.org/wiki/Energy)
    /// in [joules](https://en.wikipedia.org/wiki/Joule).
    Energy(f64),

    /// [Speed](https://en.wikipedia.org/wiki/Speed) in [m/s](https://en.wikipedia.org/wiki/Metre_per_second).
    Speed(f64),

    /// [Cloudiness](https://en.wikipedia.org/wiki/Cloud_cover), percentage.
    Cloudiness(f64),

    /// Battery relative charge, percentage.
    BatteryLife(f64),

    /// Arbitrary binary content.
    #[serde(serialize_with = "serialize_bytes", deserialize_with = "deserialize_bytes")]
    Blob(Arc<Bytes>),

    /// String value from a finite set.
    StringEnum(String),

    /// For variants that do not exist anymore but still stored in the database.
    #[serde(other)]
    Other,
}

impl AsRef<Value> for Value {
    fn as_ref(&self) -> &Self {
        &self
    }
}

fn serialize_bytes<S: Serializer>(bytes: &Arc<Bytes>, serializer: S) -> StdResult<S::Ok, S::Error> {
    serializer.serialize_bytes(bytes.as_ref())
}

fn deserialize_bytes<'de, D: Deserializer<'de>>(deserializer: D) -> StdResult<Arc<Bytes>, D::Error> {
    Ok(Arc::new(Bytes::from(
        Vec::<u8>::deserialize(deserializer).map_err(de::Error::custom)?,
    )))
}
