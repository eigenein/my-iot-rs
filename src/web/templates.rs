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
    pub fn new(sensor_title: &str, values: Vec<(DateTime<Local>, f64)>, multiplier: f64) -> Self {
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
                            "y": value * multiplier,
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
    /// Renders the value.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            // language=HTML
            Value::None => write!(f, r#"<i class="fas fa-question"></i> None"#),

            // language=HTML
            Value::Counter(count) => write!(f, r#"<i class="fas fa-sort-numeric-up-alt"></i> {}"#, count),

            // language=HTML
            Value::DataSize(byte_number) => write!(
                f,
                r#"<i class="far fa-save"></i> {}"#,
                human_format(*byte_number as f64, "B")
            ),

            // language=HTML
            Value::Text(ref text) => write!(f, r#"<i class="fas fa-quote-left"></i> {}"#, text),

            // language=HTML
            Value::Temperature(celsius) => write!(
                f,
                r#"<i class="fas fa-thermometer-half"></i> {}"#,
                human_format(*celsius, "℃")
            ),

            // language=HTML
            Value::Bft(force) => write!(f, r#"<i class="fas fa-wind"></i> {} BFT"#, force),

            // language=HTML
            Value::WindDirection(point) => write!(
                f,
                r#"<i class="fas fa-wind"></i> {}"#,
                match point {
                    PointOfTheCompass::East => "East",
                    PointOfTheCompass::EastNortheast => "East-northeast",
                    PointOfTheCompass::EastSoutheast => "East-southeast",
                    PointOfTheCompass::North => "North",
                    PointOfTheCompass::NorthNortheast => "North-northeast",
                    PointOfTheCompass::NorthNorthwest => "North-northwest",
                    PointOfTheCompass::Northeast => "Northeast",
                    PointOfTheCompass::Northwest => "Northwest",
                    PointOfTheCompass::South => "South",
                    PointOfTheCompass::SouthSoutheast => "South-southeast",
                    PointOfTheCompass::SouthSouthwest => "South-southwest",
                    PointOfTheCompass::Southeast => "Southeast",
                    PointOfTheCompass::Southwest => "Southwest",
                    PointOfTheCompass::West => "West",
                    PointOfTheCompass::WestNorthwest => "West-northwest",
                    PointOfTheCompass::WestSouthwest => "West-southwest",
                }
            ),

            // language=HTML
            Value::Rh(percentage) => write!(f, r#"<i class="fas fa-water"></i> {}%"#, percentage),

            // language=HTML
            Value::Length(meters) => write!(f, r#"<i class="fas fa-ruler"></i> {}"#, human_format(*meters, "m")),

            // language=HTML
            Value::ImageUrl(url) => write!(f, r#"<img src="{}" alt="">"#, url),

            // language=HTML
            Value::Boolean(flag) => write!(
                f,
                r#"{} <span class="is-uppercase">{}</span>"#,
                if *flag {
                    r#"<i class="fas fa-toggle-on"></i>"#
                } else {
                    r#"<i class="fas fa-toggle-off"></i>"#
                },
                if *flag { "Yes" } else { "No" }
            ),

            // language=HTML
            Value::Duration(seconds) => write!(f, r#"<i class="far fa-clock"></i> {}"#, human_format(*seconds, "s")),

            // language=HTML
            Value::RelativeIntensity(percentage) => write!(f, r#"<i class="far fa-lightbulb"></i> {}%"#, percentage),

            // language=HTML
            Value::Energy(joules) => write!(
                f,
                r#"<i class="fas fa-burn"></i> {}"#,
                human_format(*joules / JOULES_IN_WH, "Wh")
            ),

            // language=HTML
            Value::Power(watts) => write!(f, r#"<i class="fas fa-plug"></i> {}"#, human_format(*watts, "W")),

            // language=HTML
            Value::Volume(m3) => write!(f, r#"<i class="fas fa-oil-can"></i> {}"#, human_format(*m3, "㎥")),
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
}
