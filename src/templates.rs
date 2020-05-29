//! Web interface templates.

use crate::prelude::*;
use crate::settings::Settings;
use askama::Template;
use itertools::Itertools;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    #[allow(clippy::type_complexity)]
    pub actuals: Vec<(Option<String>, Vec<(Sensor, Reading)>)>,
}

impl IndexTemplate {
    pub fn new(db: &Connection, max_sensor_age_ms: i64) -> Result<Self> {
        Ok(Self {
            actuals: db
                .select_actuals(max_sensor_age_ms)?
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
        Ok(SettingsTemplate {
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

fn crate_version() -> &'static str {
    structopt::clap::crate_version!()
}

mod filters {
    use chrono::{DateTime, Local};

    pub fn format_datetime(datetime: &DateTime<Local>) -> askama::Result<String> {
        Ok(datetime.format("%b %d, %H:%M:%S").to_string())
    }
}
