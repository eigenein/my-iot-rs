//! Settings structs.

use crate::services;
use crate::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Read the settings file.
pub fn read<P: AsRef<Path>>(path: P) -> Result<Settings> {
    toml::from_str(&fs::read_to_string(path)?).map_err(Into::into)
}

/// Represents a root settings object.
#[derive(Deserialize, Debug)]
pub struct Settings {
    /// Web server port. It's used for the user interface as well as for webhooks.
    #[serde(default = "default_http_port")]
    pub http_port: u16,

    /// Maximum duration while any sensor may be inactive before it gets hidden from the UI.
    #[serde(default = "default_max_sensor_age_ms")]
    pub max_sensor_age_ms: i64,

    /// Services configuration.
    /// Each entry is a pair of service ID (defined by user) and service settings.
    /// Service ID is normally used as a sensor prefix, for instance: `service_id::service_sensor`.
    pub services: HashMap<String, Service>,
}

/// A service configuration.
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Service {
    /// Regularly emits a counter value.
    Clock(services::clock::Clock),

    /// Dutch [Buienradar](https://www.buienradar.nl/) weather service.
    Buienradar(services::buienradar::Buienradar),

    /// [Telegram bot](https://core.telegram.org/bots/api) service.
    Telegram(services::telegram::Telegram),

    /// [Lua](https://www.lua.org/) scripting.
    Lua(services::lua::Lua),

    /// Sunrise and sunset messages.
    Solar(services::solar::Solar),
}

fn default_http_port() -> u16 {
    8081
}

/// Defaults to 30 days.
fn default_max_sensor_age_ms() -> i64 {
    30 * 24 * 60 * 60 * 1000
}
