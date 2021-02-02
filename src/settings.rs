//! Settings structs.

use crate::prelude::*;
use crate::services;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Read the settings file.
pub fn read<P: AsRef<Path> + std::fmt::Debug>(paths: Vec<P>) -> Result<Settings> {
    Ok(toml::from_str(
        &paths
            .iter()
            .map(|path| {
                info!("Reading {:?}…", path);
                Ok(fs::read_to_string(path)?)
            })
            .collect::<Result<Vec<String>>>()?
            .join("\n\n"),
    )?)
}

/// Settings root.
#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Settings {
    #[serde(default)]
    pub database: DatabaseSettings,

    #[serde(default)]
    pub http: HttpSettings,

    /// Services configuration.
    ///
    /// Each entry is a pair of service ID (defined by user) and service settings.
    /// Service ID is normally used as a sensor prefix, for instance: `service_id::service_sensor`.
    #[serde(default = "HashMap::new")]
    pub services: HashMap<String, Service>,

    /// Separate section for sensitive settings.
    #[serde(default)]
    pub secrets: SecretSettings,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct SecretSettings {
    /// Optional Sentry DSN for monitoring.
    #[serde(default)]
    pub sentry_dsn: Option<String>,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct HttpSettings {
    /// Web server port. It's used for the user interface as well as for webhooks.
    #[serde(default = "default_http_port")]
    pub port: u16,

    /// Disable the web server.
    #[serde(default)]
    pub disabled: bool,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct DatabaseSettings {
    #[serde(default = "default_database_path")]
    pub path: String,
}

/// Service settings section.
#[derive(Deserialize, Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum Service {
    /// Dutch [Buienradar](https://www.buienradar.nl/) weather service.
    Buienradar(Box<services::buienradar::Buienradar>),

    /// Regularly emits a counter value.
    Clock(Box<services::clock::Clock>),

    /// [OpenWeather](https://openweathermap.org/).
    OpenWeather(Box<services::openweather::OpenWeather>),

    /// [Philips Hue bridge](https://developers.meethue.com/develop/get-started-2/)
    PhilipsHue(Box<services::philips_hue::PhilipsHue>),

    /// [Rhai](https://schungx.github.io/rhai/).
    Rhai(Box<services::rhai::Rhai>),

    /// [Ring](https://ring.com)
    Ring(Box<services::ring::Ring>),

    SimpleAnomalyDetector(Box<services::anomaly::simple_detector::SimpleAnomalyDetector>),

    /// Sunrise and sunset messages.
    Solar(Box<services::solar::Solar>),

    /// [tado°](https://www.tado.com/) API.
    Tado(Box<services::tado::Tado>),

    /// [Telegram bot](https://core.telegram.org/bots/api) service.
    Telegram(Box<services::telegram::Telegram>),

    Threshold(Box<services::threshold::Threshold>),

    /// [YouLess](https://www.youless.nl/home.html) kWh meter to ethernet bridge.
    YouLess(Box<services::youless::YouLess>),
}

impl Default for SecretSettings {
    fn default() -> Self {
        Self { sentry_dsn: None }
    }
}

impl Default for HttpSettings {
    fn default() -> Self {
        Self {
            port: default_http_port(),
            disabled: false,
        }
    }
}

impl Default for DatabaseSettings {
    fn default() -> Self {
        Self {
            path: default_database_path(),
        }
    }
}

pub fn default_http_port() -> u16 {
    8081
}

fn default_database_path() -> String {
    "my-iot.sqlite3".into()
}
