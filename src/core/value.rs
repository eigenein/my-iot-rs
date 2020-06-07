//! Implements sensor reading value.

use serde::{Deserialize, Serialize};
use uom::si::f64::*;
use uom::si::*;

/// Sensor reading value.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Value {
    /// No value.
    #[serde(rename = "N")]
    None,

    /// Generic counter.
    #[serde(rename = "C")]
    Counter(u64),

    /// Image URL.
    #[serde(rename = "IU")]
    ImageUrl(String),

    /// Boolean.
    #[serde(rename = "B")]
    Boolean(bool),

    /// Wind direction.
    #[serde(rename = "WD")]
    WindDirection(PointOfTheCompass),

    /// Size in [bytes](https://en.wikipedia.org/wiki/Byte).
    #[serde(rename = "DS")]
    DataSize(u64),

    /// [Plain text](https://en.wikipedia.org/wiki/Plain_text).
    #[serde(rename = "TEXT")]
    Text(String),

    /// [Beaufort](https://en.wikipedia.org/wiki/Beaufort_scale) wind speed.
    #[serde(rename = "BFT")]
    Bft(u8),

    /// [Relative humidity](https://en.wikipedia.org/wiki/Relative_humidity) in **percents**.
    #[serde(rename = "RH")]
    Rh(f64),

    /// [Thermodynamic temperature](https://en.wikipedia.org/wiki/Thermodynamic_temperature).
    #[serde(rename = "T")]
    Temperature(ThermodynamicTemperature),

    /// Length.
    #[serde(rename = "L")]
    Length(Length),

    /// Duration.
    #[serde(rename = "D")]
    Duration(Time),

    /// Generic intensity in percents.
    #[serde(rename = "RI")]
    RelativeIntensity(f64),
}

impl From<ThermodynamicTemperature> for Value {
    fn from(temperature: ThermodynamicTemperature) -> Self {
        Self::Temperature(temperature)
    }
}

impl From<Length> for Value {
    fn from(length: Length) -> Self {
        Self::Length(length)
    }
}

impl From<Time> for Value {
    fn from(time: Time) -> Self {
        Self::Duration(time)
    }
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
