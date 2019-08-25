//! Implements sensor reading value.

use humansize::FileSize;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Sensor reading value.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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
    /// Image URL.
    ImageUrl(String),
}

impl markup::Render for Value {
    /// Render value in HTML.
    fn render(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} {}", self.icon(), self)
    }
}

impl Value {
    /// Get [CSS color class](https://bulma.io/documentation/modifiers/color-helpers/).
    pub fn class(&self) -> &str {
        match *self {
            Value::Text(_) | Value::Counter(_) | Value::Size(_) | Value::Metres(_) | Value::ImageUrl(_) => "is-light",
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
        }
    }

    /// Get [Font Awesome](https://fontawesome.com) icon tag.
    pub fn icon(&self) -> &'static str {
        match *self {
            Value::Counter(_) => r#"<i class="fas fa-sort-numeric-up-alt"></i>"#,
            Value::Size(_) => r#"<i class="far fa-save"></i>"#,
            Value::Text(_) => r#"<i class="fas fa-quote-left"></i>"#,
            Value::Celsius(_) => r#"<i class="fas fa-thermometer-half"></i>"#,
            Value::Bft(_) => r#"<i class="fas fa-wind"></i>"#,
            Value::WindDirection(_) => r#"<i class="fas fa-wind"></i>"#,
            Value::Rh(_) => r#"<i class="fas fa-water"></i>"#,
            Value::Metres(_) => r#"<i class="fas fa-ruler"></i>"#,
            Value::ImageUrl(_) => r#"<i class="fas fa-camera"></i>"#,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Value::Counter(count) => write!(f, r"{} times", count),
            // TODO: use `human_format` instead.
            Value::Size(size) => f.write_str(&size.file_size(humansize::file_size_opts::DECIMAL).unwrap()),
            Value::Text(ref string) => write!(f, r"{}", string),
            Value::Celsius(degrees) => write!(f, r"{:.1} ℃", degrees),
            Value::Bft(bft) => write!(f, r"{} BFT", bft),
            Value::WindDirection(point) => write!(f, r"{}", point),
            Value::Rh(percent) => write!(f, r"{}%", percent),
            Value::Metres(metres) => f.write_str(&human_format(*metres, "m")),
            Value::ImageUrl(url) => write!(f, r#"<img src="{}">"#, url),
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
            // TODO
            PointOfTheCompass::SouthSouthwest => write!(f, "South-southwest"),
            // TODO
            _ => write!(f, "{:?}", self),
        }
    }
}

/// Format value to keep only 3 digits before the decimal point.
fn human_format(value: f64, unit: &str) -> String {
    match value {
        _ if value < 1e-21 => format!("{:.1} y{}", value * 1e24, unit),
        _ if value < 1e-18 => format!("{:.1} z{}", value * 1e21, unit),
        _ if value < 1e-15 => format!("{:.1} a{}", value * 1e18, unit),
        _ if value < 1e-12 => format!("{:.1} f{}", value * 1e15, unit),
        _ if value < 1e-9 => format!("{:.1} p{}", value * 1e12, unit),
        _ if value < 1e-6 => format!("{:.1} n{}", value * 1e9, unit),
        _ if value < 1e-3 => format!("{:.1} µ{}", value * 1e6, unit),
        _ if value < 1.0 => format!("{:.1} m{}", value * 1e3, unit),
        _ if value < 1e3 => format!("{:.1} {}", value, unit),
        _ if value < 1e6 => format!("{:.1} k{}", value * 1e-3, unit),
        _ if value < 1e9 => format!("{:.1} M{}", value * 1e-6, unit),
        _ if value < 1e12 => format!("{:.1} G{}", value * 1e-9, unit),
        _ if value < 1e15 => format!("{:.1} T{}", value * 1e-12, unit),
        _ if value < 1e18 => format!("{:.1} P{}", value * 1e-15, unit),
        _ if value < 1e21 => format!("{:.1} E{}", value * 1e-18, unit),
        _ if value < 1e24 => format!("{:.1} Z{}", value * 1e-21, unit),
        _ => format!("{:.1} Y{}", value * 1e-24, unit),
    }
}

#[cfg(test)]
mod tests {
    use crate::value::human_format;

    #[test]
    fn metres() {
        assert_eq!(human_format(100.0, "m"), "100.0 m");
    }

    #[test]
    fn megametres() {
        assert_eq!(human_format(12.756e6, "m"), "12.8 Mm");
    }

    #[test]
    fn millimetres() {
        assert_eq!(human_format(0.005, "m"), "5.0 mm");
    }
}
