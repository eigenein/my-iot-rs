//! Web interface templates.

use crate::format::human_format;
use crate::prelude::*;
use crate::settings::Settings;
use askama::Template;
use itertools::Itertools;
use std::collections::HashMap;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub temperature: Value,
    pub feel_temperature: Value,
}

impl Default for IndexTemplate {
    fn default() -> Self {
        Self {
            temperature: Value::None,
            feel_temperature: Value::None,
        }
    }
}

impl IndexTemplate {
    pub fn new(db: &Connection, settings: &Settings) -> Result<Self> {
        let mut template = Self::default();

        let actuals: HashMap<String, Reading> = db
            .select_actuals()?
            .into_iter()
            .map(|(sensor, reading)| (sensor.id, reading))
            .collect();

        let dashboard = &settings.dashboard;
        template.temperature = Self::get_dashboard_value(&actuals, &dashboard.temperature_sensor);
        template.feel_temperature = Self::get_dashboard_value(&actuals, &dashboard.feel_temperature_sensor);

        Ok(template)
    }

    /// Returns actual sensor value or `Value::None` otherwise.
    fn get_dashboard_value(actuals: &HashMap<String, Reading>, sensor_id: &Option<String>) -> Value {
        sensor_id
            .as_ref()
            .and_then(|sensor_id| actuals.get(sensor_id))
            .map_or(Value::None, |reading| reading.value.clone())
    }
}

#[derive(Template)]
#[template(path = "sensors.html")]
pub struct SensorsTemplate {
    #[allow(clippy::type_complexity)]
    pub actuals: Vec<(Option<String>, Vec<(Sensor, Reading)>)>,
}

impl SensorsTemplate {
    pub fn new(db: &Connection) -> Result<Self> {
        Ok(Self {
            actuals: db
                .select_actuals()?
                .into_iter()
                .group_by(|(sensor, _)| sensor.room_title.clone())
                .into_iter()
                .map(|(room_title, group)| (room_title, group.collect_vec()))
                .collect_vec(),
        })
    }
}

#[derive(Template)]
#[template(path = "settings.html")]
pub struct SettingsTemplate {
    pub settings: String,
}

impl SettingsTemplate {
    pub fn new(settings: &Settings) -> Result<Self> {
        Ok(Self {
            settings: toml::to_string_pretty(&settings)?,
        })
    }
}

#[derive(Template)]
#[template(path = "sensor.html")]
pub struct SensorTemplate {
    pub sensor: Sensor,
    pub reading: Reading,
}

impl SensorTemplate {
    pub fn new(sensor: Sensor, reading: Reading) -> Self {
        Self { sensor, reading }
    }
}

/// Navigation bar.
#[derive(Template)]
#[template(path = "partials/navbar.html")]
pub struct NavbarPartialTemplate<'a> {
    pub selected_item: &'a str,
}

impl<'a> NavbarPartialTemplate<'a> {
    pub fn new(selected_item: &'a str) -> Self {
        NavbarPartialTemplate { selected_item }
    }
}

/// `Value` renderer.
#[derive(Template)]
#[template(path = "partials/value.html")]
pub struct ValuePartialTemplate<'a> {
    pub value: &'a Value,
}

impl<'a> ValuePartialTemplate<'a> {
    pub fn new(value: &'a Value) -> Self {
        ValuePartialTemplate { value }
    }
}

/// Dashboard tile.
#[derive(Template)]
#[template(path = "partials/sensor_tile.html")]
pub struct SensorTilePartialTemplate<'a> {
    pub sensor: &'a Sensor,

    /// The actual reading.
    pub reading: &'a Reading,
}

impl<'a> SensorTilePartialTemplate<'a> {
    pub fn new(sensor: &'a Sensor, reading: &'a Reading) -> Self {
        SensorTilePartialTemplate { sensor, reading }
    }
}

impl Value {
    /// Get whether value could be rendered inline.
    pub fn is_inline(&self) -> bool {
        match self {
            Value::ImageUrl(_) => false,
            _ => true,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::None => write!(f, "None"),
            Value::Counter(value) => write!(f, r"{}", value),
            Value::DataSize(value) => f.write_str(&human_format(*value as f64, "B")),
            Value::Text(ref value) => write!(f, r"{}", value),
            Value::Temperature(value) => f.write_str(&human_format(*value, "℃")),
            Value::Bft(value) => write!(f, r"{} BFT", value),
            Value::WindDirection(value) => match value {
                PointOfTheCompass::East => write!(f, "East"),
                PointOfTheCompass::EastNortheast => write!(f, "East-northeast"),
                PointOfTheCompass::EastSoutheast => write!(f, "East-southeast"),
                PointOfTheCompass::North => write!(f, "North"),
                PointOfTheCompass::NorthNortheast => write!(f, "North-northeast"),
                PointOfTheCompass::NorthNorthwest => write!(f, "North-northwest"),
                PointOfTheCompass::Northeast => write!(f, "Northeast"),
                PointOfTheCompass::Northwest => write!(f, "Northwest"),
                PointOfTheCompass::South => write!(f, "South"),
                PointOfTheCompass::SouthSoutheast => write!(f, "South-southeast"),
                PointOfTheCompass::SouthSouthwest => write!(f, "South-southwest"),
                PointOfTheCompass::Southeast => write!(f, "Southeast"),
                PointOfTheCompass::Southwest => write!(f, "Southwest"),
                PointOfTheCompass::West => write!(f, "West"),
                PointOfTheCompass::WestNorthwest => write!(f, "West-northwest"),
                PointOfTheCompass::WestSouthwest => write!(f, "West-southwest"),
            },
            Value::Rh(value) => write!(f, "{}%", value),
            Value::Length(value) => f.write_str(&human_format(*value, "m")),
            Value::ImageUrl(value) => write!(f, r#"<img src="{}">"#, value),
            Value::Boolean(value) => write!(
                f,
                r#"<span class="is-uppercase">{}</span>"#,
                if *value { "Yes" } else { "No" }
            ),
            Value::Duration(value) => f.write_str(&human_format(*value, "s")),
            Value::RelativeIntensity(value) => write!(f, "{}%", value),
            Value::Energy(value) => f.write_str(&human_format(*value, "J")),
            Value::Power(value) => f.write_str(&human_format(*value, "W")),
            Value::Volume(value) => f.write_str(&human_format(*value, "㎥")),
        }
    }
}

impl Sensor {
    /// Returns the sensor title or the sensor ID otherwise.
    pub fn title(&self) -> String {
        self.title.as_ref().unwrap_or(&self.id).into()
    }

    /// Returns the room title or the default one otherwise.
    pub fn room_title(&self) -> String {
        match &self.room_title {
            Some(title) => title.clone(),
            None => "No Room".into(),
        }
    }
}

/// Wraps `crate_version!` in order to include it in a template.
fn crate_version() -> &'static str {
    structopt::clap::crate_version!()
}

/// Custom [Askama template filters](https://docs.rs/askama/0.9.0/askama/index.html#filters).
mod filters {
    use crate::prelude::*;

    pub fn format_datetime(datetime: &DateTime<Local>) -> askama::Result<String> {
        Ok(datetime.format("%b %d, %H:%M:%S").to_string())
    }

    /// Returns a [column size](https://bulma.io/documentation/columns/sizes/) suitable to fit the value.
    pub fn column_width(value: &Value) -> askama::Result<&'static str> {
        Ok(match value {
            Value::ImageUrl(_) => "is-4",
            _ => "is-2",
        })
    }

    /// Returns a [color class](https://bulma.io/documentation/modifiers/color-helpers/) to display the value.
    pub fn color_class(value: &Value) -> askama::Result<&'static str> {
        Ok(match *value {
            Value::Bft(number) => match number {
                0 => "is-light",
                1..=3 => "is-success",
                4..=5 => "is-warning",
                _ => "is-danger",
            },
            Value::Temperature(value) => match value {
                _ if value < -5.0 + 273.15 => "is-link",
                _ if value < 5.0 + 273.15 => "is-info",
                _ if value < 15.0 + 273.15 => "is-primary",
                _ if value < 25.0 + 273.15 => "is-success",
                _ if value < 30.0 + 273.15 => "is-warning",
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
            Value::RelativeIntensity(value) => match value {
                _ if value < 15.0 => "is-link",
                _ if value < 30.0 => "is-info",
                _ if value < 50.0 => "is-primary",
                _ if value < 70.0 => "is-success",
                _ if value < 90.0 => "is-warning",
                _ => "is-danger",
            },
            _ => "is-light",
        })
    }

    /// Returns a [Font Awesome](https://fontawesome.com) icon tag for the value.
    pub fn icon(value: &Value) -> askama::Result<&'static str> {
        Ok(match *value {
            Value::Bft(_) => r#"<i class="fas fa-wind"></i>"#,
            Value::Counter(_) => r#"<i class="fas fa-sort-numeric-up-alt"></i>"#,
            Value::DataSize(_) => r#"<i class="far fa-save"></i>"#,
            Value::Length(_) => r#"<i class="fas fa-ruler"></i>"#,
            Value::Rh(_) => r#"<i class="fas fa-water"></i>"#,
            Value::Temperature(_) => r#"<i class="fas fa-thermometer-half"></i>"#,
            Value::Text(_) => r#"<i class="fas fa-quote-left"></i>"#,
            Value::WindDirection(_) => r#"<i class="fas fa-wind"></i>"#,
            Value::Boolean(value) => {
                if value {
                    r#"<i class="fas fa-toggle-on"></i>"#
                } else {
                    r#"<i class="fas fa-toggle-off"></i>"#
                }
            }
            Value::Duration(_) => r#"<i class="far fa-clock"></i>"#,
            Value::ImageUrl(_) | Value::None => "",
            Value::RelativeIntensity(_) => r#"<i class="far fa-lightbulb"></i>"#,
            Value::Energy(_) => r#"<!-- TODO -->"#,
            Value::Power(_) => r#"<!-- TODO -->"#,
            Value::Volume(_) => r#"<!-- TODO -->"#,
        })
    }
}
