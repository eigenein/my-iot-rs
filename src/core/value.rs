//! Implements sensor reading value.

use serde::{Deserialize, Serialize};

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

    /// [Thermodynamic temperature](https://en.wikipedia.org/wiki/Thermodynamic_temperature) in Celsius.
    Temperature(f64),

    /// Length in meters.
    Length(f64),

    /// Duration in seconds.
    Duration(f64),

    /// Generic intensity in percents.
    RelativeIntensity(f64),
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
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Rh(value) | Value::RelativeIntensity(value) | Value::Temperature(value) => Some(*value),
            _ => None,
        }
    }
}
