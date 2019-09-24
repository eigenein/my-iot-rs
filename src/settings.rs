//! Settings structs.

use crate::services;
use crate::Result;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

/// Read the settings file.
pub fn read<P: AsRef<Path>>(path: P) -> Result<Settings> {
    toml::from_str(&fs::read_to_string(path)?).map_err(Into::into)
}

/// Represents a root settings object.
#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    /// Web server port. It's used for the user interface as well as for webhooks.
    #[serde(default = "default_http_port")]
    pub http_port: u16,

    /// Services configuration.
    /// Each entry is a pair of service ID (defined by user) and service settings.
    /// Service ID is normally used as a sensor prefix, for instance: `service_id:service_sensor`.
    pub services: HashMap<String, ServiceSettings>,

    /// Service IDs to disable.
    #[serde(default = "HashSet::new")]
    pub disabled_services: HashSet<String>,
}

/// A service configuration.
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum ServiceSettings {
    /// Regularly emits a counter value.
    Clock(services::clock::Settings),
    /// Regularly emits database information.
    Db(services::db::Settings),
    /// Dutch [Buienradar](https://www.buienradar.nl/) weather service.
    Buienradar(services::buienradar::Settings),
    /// Nest API.
    Nest(services::nest::Settings),
    /// Automation.
    Automator(services::automator::Settings),
    /// [Telegram bot](https://core.telegram.org/bots/api) service.
    Telegram(services::telegram::Settings),
}

fn default_http_port() -> u16 {
    8081
}
