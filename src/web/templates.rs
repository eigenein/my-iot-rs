//! Web interface templates.

use crate::format::human_format;
use crate::prelude::*;
use crate::web::rocket_uri_macro_get_sensor_json;
use askama::Template;
use rocket::uri;
use serde_json::json;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    #[allow(clippy::type_complexity)]
    pub actuals: Vec<(String, Vec<(Sensor, Reading)>)>,

    pub message_count: u64,
}

#[derive(Template)]
#[template(path = "settings.html")]
pub struct SettingsTemplate {
    pub settings: String,
    pub message_count: u64,
}

#[derive(Template)]
#[template(path = "sensor.html")]
pub struct SensorTemplate {
    pub sensor: Sensor,
    pub reading: Reading,

    /// Stringified sensor chart, may be empty.
    pub chart: String,

    pub message_count: u64,

    /// Chart period.
    pub minutes: i64,
}

/// Navigation bar.
#[derive(Template)]
#[template(path = "partials/navbar.html")]
struct NavbarPartialTemplate<'a> {
    selected_item: &'a str,
}

impl<'a> NavbarPartialTemplate<'a> {
    fn new(selected_item: &'a str) -> Self {
        NavbarPartialTemplate { selected_item }
    }
}

/// `Value` renderer.
#[derive(Template)]
#[template(path = "partials/value.html")]
struct ValuePartialTemplate<'a> {
    value: &'a Value,
}

impl<'a> ValuePartialTemplate<'a> {
    fn new(value: &'a Value) -> Self {
        ValuePartialTemplate { value }
    }
}

/// Dashboard tile.
#[derive(Template)]
#[template(path = "partials/sensor_tile.html")]
struct SensorTilePartialTemplate<'a> {
    sensor: &'a Sensor,

    /// The latest reading.
    reading: &'a Reading,
}

impl<'a> SensorTilePartialTemplate<'a> {
    fn new(sensor: &'a Sensor, reading: &'a Reading) -> Self {
        SensorTilePartialTemplate { sensor, reading }
    }
}

#[derive(Template)]
#[template(path = "partials/chart.html")]
pub struct F64ChartPartialTemplate {
    chart: serde_json::Value,
}

impl F64ChartPartialTemplate {
    pub fn new(sensor_title: &str, values: Vec<(DateTime<Local>, f64)>) -> Self {
        F64ChartPartialTemplate {
            chart: json!({
                "type": "line",
                "options": chart_options(sensor_title),
                "data": {
                    "datasets": [{
                        "label": sensor_title,
                        "borderColor": "#209CEE",
                        "fill": false,
                        "data": values.iter().map(|(timestamp, value)| json!({
                            "x": timestamp.timestamp_millis(),
                            "y": value,
                        })).collect::<serde_json::Value>(),
                    }],
                },
            }),
        }
    }
}

fn chart_time_format() -> serde_json::Value {
    json!({
        "tooltipFormat": "MMM DD HH:mm:ss.SSS",
        "displayFormats": {
            "millisecond": "HH:mm:ss.SSS",
            "second": "HH:mm:ss",
            "minute": "HH:mm",
            "hour": "HH",
        },
    })
}

fn chart_options(sensor_title: &str) -> serde_json::Value {
    json!({
        "animation": {"duration": 0},
        "maintainAspectRatio": false,
        "scales": {
            "xAxes": [{
                "type": "time",
                "display": true,
                "scaleLabel": {"display": true, "labelString": "Timestamp"},
                "time": chart_time_format(),
                "ticks": {"autoSkipPadding": 10},
            }],
            "yAxes": [{
                "display": true,
                "scaleLabel": {"display": true, "labelString": sensor_title},
            }],
        },
        "tooltips": {"intersect": false},
        "elements": {"point": {"radius": 0}},
    })
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

    pub fn slug<S: AsRef<str>>(string: S) -> askama::Result<String> {
        Ok(slug::slugify(string))
    }

    pub fn format_datetime(datetime: &DateTime<Local>) -> askama::Result<String> {
        Ok(datetime.format("%b %d, %H:%M:%S").to_string())
    }

    /// Returns a [column size](https://bulma.io/documentation/columns/sizes/) suitable to fit the value.
    pub fn column_width(value: &Value) -> askama::Result<&'static str> {
        Ok(match value {
            Value::ImageUrl(_) => "is-4",
            _ => "is-3",
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
            Value::Temperature(celsius) => match celsius {
                _ if celsius < -5.0 => "is-link",
                _ if celsius < 5.0 => "is-info",
                _ if celsius < 15.0 => "is-primary",
                _ if celsius < 25.0 => "is-success",
                _ if celsius < 30.0 => "is-warning",
                _ => "is-danger",
            },
            Value::WindDirection(_) => "is-light",
            Value::Rh(percentage) => match percentage {
                _ if percentage < 25.0 => "is-link",
                _ if percentage < 30.0 => "is-info",
                _ if percentage < 45.0 => "is-primary",
                _ if percentage < 55.0 => "is-success",
                _ if percentage < 60.0 => "is-warning",
                _ => "is-danger",
            },
            Value::Boolean(value) => {
                if value {
                    "is-success"
                } else {
                    "is-danger"
                }
            }
            Value::RelativeIntensity(percentage) => match percentage {
                _ if percentage < 15.0 => "is-link",
                _ if percentage < 30.0 => "is-info",
                _ if percentage < 50.0 => "is-primary",
                _ if percentage < 70.0 => "is-success",
                _ if percentage < 90.0 => "is-warning",
                _ => "is-danger",
            },
            Value::Power(watts) => match watts {
                _ if watts <= 0.0 => "is-success",
                _ if watts <= 4000.0 => "is-warning",
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
            Value::ImageUrl(_) | Value::None => r#"<i class="fas fa-question"></i>"#,
            Value::RelativeIntensity(_) => r#"<i class="far fa-lightbulb"></i>"#,
            Value::Energy(_) => r#"<i class="fas fa-burn"></i>"#,
            Value::Power(_) => r#"<i class="fas fa-plug"></i>"#,
            Value::Volume(_) => r#"<i class="fas fa-oil-can"></i>"#,
        })
    }
}
