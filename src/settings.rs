//! # Settings
//!
//! My IoT is configured with a single YAML file
//! which must contain exactly one [`Settings`](struct.Settings.html) object.
//!
//! ## Example
//!
//! ```yaml
//! http_port: 8080
//! services:
//!   heartbeat_clock:
//!     Clock:
//!       interval_ms: 2000
//!       suffix: heartbeat
//!   database:
//!     Db:
//!       interval_ms: 2000
//!   schiphol_buienradar:
//!     Buienradar:
//!       station_id: 6240
//! ```

use crate::services;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;

/// Read the settings file.
pub fn read() -> Settings {
    serde_yaml::from_reader(File::open("settings.yml").unwrap()).unwrap()
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
}

/// A service configuration.
#[derive(Deserialize, Debug, Clone)]
pub enum ServiceSettings {
    /// Regularly emits a counter value.
    Clock(services::clock::ClockSettings),
    /// Regularly emits database information.
    Db(services::db::DbSettings),
    /// Dutch [Buienradar](https://www.buienradar.nl/) weather service.
    Buienradar(services::buienradar::BuienradarSettings),
}

fn default_http_port() -> u16 {
    8081
}
