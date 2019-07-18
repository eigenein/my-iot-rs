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
//! - Clock:
//!     interval_ms: 2000
//!     suffix: heartbeat
//! - Db:
//!     interval_ms: 2000
//! - Buienradar:
//!     station_id: 6240
//! ```

use crate::services;
use serde::Deserialize;
use std::fs::File;

/// Read the settings file.
pub fn read() -> Settings {
    serde_yaml::from_reader(File::open("settings.yml").unwrap()).unwrap()
}

/// Represents a root settings object.
#[derive(Deserialize, Debug)]
pub struct Settings {
    /// Web server port. It's used for the user interface as well as for webhooks.
    pub http_port: Option<u16>,
    /// Configured services.
    pub services: Vec<ServiceSettings>,
}

/// A service configuration.
#[derive(Deserialize, Debug)]
pub enum ServiceSettings {
    /// Regularly emits a counter value.
    Clock(services::clock::ClockSettings),
    /// Regularly emits database information.
    Db(services::db::DbSettings),
    /// Dutch [Buienradar](https://www.buienradar.nl/) weather service.
    Buienradar(services::buienradar::BuienradarSettings),
}
