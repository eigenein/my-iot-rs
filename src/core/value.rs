//! Implements sensor reading value.

use serde::{Deserialize, Serialize};

use crate::prelude::*;
use bytes::Bytes;
use std::sync::Arc;

/// Sensor reading value.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value")]
pub enum Value {
    /// No value.
    None,

    /// Generic counter.
    Counter(u64),

    /// Image URL.
    ImageUrl(String),

    /// Boolean.
    Boolean(bool),

    /// Wind direction.
    WindDirection(PointOfTheCompass),

    /// Size in [bytes](https://en.wikipedia.org/wiki/Byte).
    DataSize(u64),

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

    /// Binary content. Isn't stored in a database at the moment.
    /// Perhaps, I should write a custom serialization for this particular variant.
    #[serde(skip)]
    Blob(Arc<Bytes>),
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl AsRef<Value> for Value {
    fn as_ref(&self) -> &Self {
        &self
    }
}

/// [Points of the compass](https://en.wikipedia.org/wiki/Points_of_the_compass).
/// Provides the common aliases in English and Dutch.
#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum PointOfTheCompass {
    #[serde(alias = "N")]
    North,

    #[serde(alias = "NNE", alias = "NNO")]
    NorthNortheast,

    #[serde(alias = "NE", alias = "NO")]
    Northeast,

    #[serde(alias = "ENE", alias = "ONO")]
    EastNortheast,

    #[serde(alias = "E", alias = "O")]
    East,

    #[serde(alias = "ESE", alias = "OZO")]
    EastSoutheast,

    #[serde(alias = "SE", alias = "ZO")]
    Southeast,

    #[serde(alias = "SSE", alias = "ZZO")]
    SouthSoutheast,

    #[serde(alias = "S", alias = "Z")]
    South,

    #[serde(alias = "SSW", alias = "ZZW")]
    SouthSouthwest,

    #[serde(alias = "SW", alias = "ZW")]
    Southwest,

    #[serde(alias = "WSW", alias = "WZW")]
    WestSouthwest,

    #[serde(alias = "W")]
    West,

    #[serde(alias = "WNW")]
    WestNorthwest,

    #[serde(alias = "NW")]
    Northwest,

    #[serde(alias = "NNW")]
    NorthNorthwest,
}

impl Value {
    /// Builds a `Value` instance from [kilowatt-hours](https://en.wikipedia.org/wiki/Kilowatt-hour).
    #[inline(always)]
    pub fn from_kwh(kwh: f64) -> Self {
        Value::Energy(kwh * 1000.0 * JOULES_IN_WH)
    }

    #[inline(always)]
    pub fn from_mm(mm: f64) -> Self {
        Value::Length(mm / 1000.0)
    }
}
