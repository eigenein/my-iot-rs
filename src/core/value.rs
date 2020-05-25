//! Implements sensor reading value.

use crate::format::human_format;
use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use uom::fmt::DisplayStyle::Abbreviation;
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

impl Value {
    /// Get [CSS color class](https://bulma.io/documentation/modifiers/color-helpers/).
    pub fn class(&self) -> &str {
        // TODO: move to templates.
        match *self {
            Value::Bft(number) => match number {
                0 => "is-light",
                1..=3 => "is-success",
                4..=5 => "is-warning",
                _ => "is-danger",
            },
            Value::Temperature(value) => match value {
                _ if value.value < -5.0 + 273.15 => "is-link",
                _ if value.value < 5.0 + 273.15 => "is-info",
                _ if value.value < 15.0 + 273.15 => "is-primary",
                _ if value.value < 25.0 + 273.15 => "is-success",
                _ if value.value < 30.0 + 273.15 => "is-warning",
                _ => "is-danger",
            },
            Value::WindDirection(_) => "is-light",
            Value::Rh(value) => match value {
                _ if value < 25.0 => "is-link",
                _ if value < 30.0 => "is-info",
                _ if value < 45.0 => "is-primary",
                _ if value < 55.0 => "is-success",
                _ if value < 60.0 => "is-warning",
                _ => "is-danger",
            },
            Value::Boolean(value) => {
                if value {
                    "is-success"
                } else {
                    "is-danger"
                }
            }
            _ => "is-light",
        }
    }

    /// Get [Font Awesome](https://fontawesome.com) icon tag.
    pub fn icon(&self) -> Result<&'static str> {
        // TODO: move to templates.
        match *self {
            Value::Bft(_) => Ok(r#"<i class="fas fa-wind"></i>"#),
            Value::Counter(_) => Ok(r#"<i class="fas fa-sort-numeric-up-alt"></i>"#),
            Value::DataSize(_) => Ok(r#"<i class="far fa-save"></i>"#),
            Value::Length(_) => Ok(r#"<i class="fas fa-ruler"></i>"#),
            Value::Rh(_) => Ok(r#"<i class="fas fa-water"></i>"#),
            Value::Temperature(_) => Ok(r#"<i class="fas fa-thermometer-half"></i>"#),
            Value::Text(_) => Ok(r#"<i class="fas fa-quote-left"></i>"#),
            Value::WindDirection(_) => Ok(r#"<i class="fas fa-wind"></i>"#),
            Value::Boolean(value) => Ok(if value {
                r#"<i class="fas fa-toggle-on"></i>"#
            } else {
                r#"<i class="fas fa-toggle-off"></i>"#
            }),
            Value::Duration(_) => Ok(r#"<i class="far fa-clock"></i>"#),
            Value::ImageUrl(_) | Value::None => Ok(""),
        }
    }
}

// TODO: move to templates.
impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Value::None => write!(f, "None"),
            Value::Counter(count) => write!(f, r"{}", count),
            Value::DataSize(size) => f.write_str(&human_format(*size as f64, "B")),
            Value::Text(ref string) => write!(f, r"{}", string),
            Value::Temperature(temperature) => write!(
                f,
                r"{:.1}",
                temperature.into_format_args(thermodynamic_temperature::degree_celsius, Abbreviation),
            ),
            Value::Bft(bft) => write!(f, r"{} BFT", bft),
            Value::WindDirection(point) => write!(f, r"{}", point),
            Value::Rh(percent) => write!(f, "{}%", percent),
            Value::Length(length) => write!(f, "{}", length.into_format_args(length::meter, Abbreviation)),
            Value::ImageUrl(url) => write!(f, r#"<img src="{}">"#, url),
            Value::Boolean(value) => write!(
                f,
                r#"<span class="is-uppercase">{}</span>"#,
                if *value { "Yes" } else { "No" }
            ),
            Value::Duration(time) => write!(f, "{}", time.into_format_args(time::second, Abbreviation)),
        }
    }
}

/// [Points of the compass](https://en.wikipedia.org/wiki/Points_of_the_compass).
#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum PointOfTheCompass {
    /// N
    North,
    /// NNE
    NorthNortheast,
    /// NE
    Northeast,
    /// ENE
    EastNortheast,
    /// E
    East,
    /// ESE
    EastSoutheast,
    /// SE
    Southeast,
    /// SSE
    SouthSoutheast,
    /// S
    South,
    /// SSW
    SouthSouthwest,
    /// SW
    Southwest,
    /// WSW
    WestSouthwest,
    /// W
    West,
    /// WNW
    WestNorthwest,
    /// NW
    Northwest,
    /// NNW
    NorthNorthwest,
}

// TODO: move to templates.
impl Display for PointOfTheCompass {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            PointOfTheCompass::North => write!(f, "North"),
            PointOfTheCompass::NorthNortheast => write!(f, "North-northeast"),
            PointOfTheCompass::Northeast => write!(f, "Northeast"),
            PointOfTheCompass::EastNortheast => write!(f, "East-northeast"),
            PointOfTheCompass::East => write!(f, "East"),
            PointOfTheCompass::EastSoutheast => write!(f, "East-southeast"),
            PointOfTheCompass::Southeast => write!(f, "Southeast"),
            PointOfTheCompass::SouthSoutheast => write!(f, "South-southeast"),
            PointOfTheCompass::South => write!(f, "South"),
            PointOfTheCompass::SouthSouthwest => write!(f, "South-southwest"),
            // TODO
            _ => write!(f, "{:?}", self),
        }
    }
}
