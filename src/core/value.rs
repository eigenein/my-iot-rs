//! Implements sensor reading value.

use crate::format::human_format;
use failure::{format_err, Error};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Sensor reading value.
#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(untagged)]
pub enum Value {
    /// No value.
    None,

    /// Generic counter.
    Counter(u64),

    /// Size in [bytes](https://en.wikipedia.org/wiki/Byte).
    Size(u64),

    /// [Plain text](https://en.wikipedia.org/wiki/Plain_text).
    Text(String),

    /// [Celsius](https://en.wikipedia.org/wiki/Celsius) temperature.
    Celsius(f64),

    /// [Beaufort](https://en.wikipedia.org/wiki/Beaufort_scale) wind speed.
    Bft(u8),

    /// Wind direction.
    WindDirection(PointOfTheCompass),

    /// Length in [metres](https://en.wikipedia.org/wiki/Metre).
    Metres(f64),

    /// [Relative humidity](https://en.wikipedia.org/wiki/Relative_humidity) in percents.
    Rh(f64),

    /// Image URL.
    ImageUrl(String),

    /// Boolean.
    Boolean(bool),
}

// TODO: move rendering to a separate module.
impl markup::Render for Value {
    /// Render value in HTML.
    fn render(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} {}", self.icon().unwrap_or(""), self)
    }
}

impl Value {
    /// Get [CSS color class](https://bulma.io/documentation/modifiers/color-helpers/).
    pub fn class(&self) -> &str {
        match *self {
            Value::Bft(number) => match number {
                0 => "is-light",
                1..=3 => "is-success",
                4..=5 => "is-warning",
                _ => "is-danger",
            },
            Value::Celsius(value) => match value {
                _ if value < -5.0 => "is-link",
                _ if value < 5.0 => "is-info",
                _ if value < 15.0 => "is-primary",
                _ if value < 25.0 => "is-success",
                _ if value < 30.0 => "is-warning",
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
    pub fn icon(&self) -> Result<&'static str, Error> {
        match *self {
            Value::Counter(_) => Ok(r#"<i class="fas fa-sort-numeric-up-alt"></i>"#),
            Value::Size(_) => Ok(r#"<i class="far fa-save"></i>"#),
            Value::Text(_) => Ok(r#"<i class="fas fa-quote-left"></i>"#),
            Value::Celsius(_) => Ok(r#"<i class="fas fa-thermometer-half"></i>"#),
            Value::Bft(_) => Ok(r#"<i class="fas fa-wind"></i>"#),
            Value::WindDirection(_) => Ok(r#"<i class="fas fa-wind"></i>"#),
            Value::Rh(_) => Ok(r#"<i class="fas fa-water"></i>"#),
            Value::Metres(_) => Ok(r#"<i class="fas fa-ruler"></i>"#),
            Value::Boolean(value) => Ok(if value {
                r#"<i class="fas fa-toggle-on"></i>"#
            } else {
                r#"<i class="fas fa-toggle-off"></i>"#
            }),
            _ => Err(format_err!("value has no icon")),
        }
    }

    /// Get whether value could be rendered inline.
    pub fn is_inline(&self) -> bool {
        match self {
            Value::ImageUrl(_) => false,
            _ => true,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Value::None => Ok(()),
            Value::Counter(count) => write!(f, r"{} times", count),
            Value::Size(size) => f.write_str(&human_format(*size as f64, "B")),
            Value::Text(ref string) => write!(f, r"{}", string),
            Value::Celsius(degrees) => write!(f, r"{:.1} ℃", degrees),
            Value::Bft(bft) => write!(f, r"{} BFT", bft),
            Value::WindDirection(point) => write!(f, r"{}", point),
            Value::Rh(percent) => write!(f, r"{}%", percent),
            Value::Metres(metres) => f.write_str(&human_format(*metres, "m")),
            Value::ImageUrl(url) => write!(f, r#"<img src="{}">"#, url),
            Value::Boolean(value) => write!(
                f,
                r#"<span class="is-uppercase">{}</span>"#,
                if *value { "Yes" } else { "No" }
            ),
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
