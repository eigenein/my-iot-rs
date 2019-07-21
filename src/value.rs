//! Implements sensor measurement value.

use humansize::FileSize;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Sensor measurement value.
#[derive(Debug, Serialize, Deserialize)]
pub enum Value {
    /// Generic counter.
    Counter(u64),
    /// Size in bytes.
    Size(u64),
    /// Plain text.
    Text(String),
    /// Celsius temperature.
    Celsius(f64),
    /// Beaufort wind speed.
    Bft(u32),
    /// Wind direction.
    WindDirection(PointOfTheCompass),
}

impl markup::Render for Value {
    /// Render value in HTML.
    fn render(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // TODO: perhaps I need to implement `icon()` and `std::fmt::Display` for `Value` to separate
        // TODO: icon and text rendering.
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
            Value::WindDirection(point) => write!(f, r#"<i class="fas fa-wind"></i> {}"#, point), // TODO
        }
    }
}

impl Value {
    /// Retrieve [CSS color class](https://bulma.io/documentation/modifiers/color-helpers/).
    pub fn class(&self) -> &str {
        match *self {
            Value::Text(_) | Value::Counter(_) | Value::Size(_) => "is-light",
            Value::Bft(_) => "is-light", // TODO
            Value::Celsius(value) => match value {
                // TODO
                _ if 5.0 <= value && value < 15.0 => "is-primary",
                _ if 15.0 <= value && value < 25.0 => "is-success",
                _ => panic!("value {} is not covered", value),
            },
            Value::WindDirection(_) => "is-light",
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
            // TODO
            PointOfTheCompass::SouthSouthwest => write!(f, "South-southwest"),
            _ => write!(f, "{:?}", self),
        }
    }
}
