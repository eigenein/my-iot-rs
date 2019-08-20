//! Implements sensor reading value.

use humansize::FileSize;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Sensor reading value.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Value {
    /// Generic counter.
    Counter(u64),
    /// Size in [bytes](https://en.wikipedia.org/wiki/Byte).
    Size(u64),
    /// [Plain text](https://en.wikipedia.org/wiki/Plain_text).
    Text(String),
    /// [Celsius](https://en.wikipedia.org/wiki/Celsius) temperature.
    Celsius(f64),
    /// [Beaufort](https://en.wikipedia.org/wiki/Beaufort_scale) wind speed.
    Bft(u32),
    /// Wind direction.
    WindDirection(PointOfTheCompass),
    /// Length in [metres](https://en.wikipedia.org/wiki/Metre).
    Metres(f64),
    /// [Relative humidity](https://en.wikipedia.org/wiki/Relative_humidity) in percents.
    Rh(f64),
}

impl markup::Render for Value {
    /// Render value in HTML.
    fn render(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // TODO: implement `icon()` and `std::fmt::Display` for `Value` to separate icon and text rendering.
        match *self {
            Value::Counter(count) => write!(f, r#"<i class="fas fa-sort-numeric-up-alt"></i> {} times"#, count),
            Value::Size(size) => write!(
                f,
                r#"<i class="far fa-save"></i> {}"#,
                size.file_size(humansize::file_size_opts::DECIMAL).unwrap()
            ),
            Value::Text(ref string) => write!(f, r#"<i class="fas fa-quote-left"></i> {}"#, string),
            Value::Celsius(degrees) => write!(f, r#"<i class="fas fa-thermometer-half"></i> {:.1} ℃"#, degrees),
            Value::Bft(bft) => write!(f, r#"<i class="fas fa-wind"></i> {} BFT"#, bft),
            Value::WindDirection(point) => write!(f, r#"<i class="fas fa-wind"></i> {}"#, point),
            Value::Rh(percent) => write!(f, r#"<i class="fas fa-wateer"></i> {}%"#, percent),
            _ => unimplemented!(),
        }
    }
}

impl Value {
    /// Retrieve [CSS color class](https://bulma.io/documentation/modifiers/color-helpers/).
    pub fn class(&self) -> &str {
        match *self {
            Value::Text(_) | Value::Counter(_) | Value::Size(_) => "is-light",
            Value::Bft(number) => match number {
                0 => "is-light",
                1..=3 => "is-success",
                4..=5 => "is-warning",
                _ => "is-danger",
            },
            Value::Celsius(value) => match value {
                _ if value < -5.0 => "is-link",
                _ if -5.0 <= value && value < 5.0 => "is-info",
                _ if 5.0 <= value && value < 15.0 => "is-primary",
                _ if 15.0 <= value && value < 25.0 => "is-success",
                _ if 25.0 <= value && value < 30.0 => "is-warning",
                _ if 30.0 <= value => "is-danger",
                _ => unreachable!(),
            },
            Value::WindDirection(_) => "is-light",
            Value::Rh(value) => match value {
                _ if value < 25.0 => "is-link",
                _ if 25.0 <= value && value < 30.0 => "is-info",
                _ if 30.0 <= value && value < 45.0 => "is-primary",
                _ if 45.0 <= value && value < 55.0 => "is-success",
                _ if 55.0 <= value && value < 60.0 => "is-warning",
                _ if 60.0 <= value => "is-danger",
                _ => unreachable!(),
            },
            _ => unimplemented!(),
        }
    }
}

/// [Points of the compass](https://en.wikipedia.org/wiki/Points_of_the_compass).
#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
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
            // TODO
            PointOfTheCompass::SouthSouthwest => write!(f, "South-southwest"),
            // TODO
            _ => write!(f, "{:?}", self),
        }
    }
}
