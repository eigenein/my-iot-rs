//! Settings.
use crate::services;
use serde::Deserialize;
use std::fs::File;

/// Read the settings file.
pub fn read() -> Settings {
    serde_yaml::from_reader(File::open("settings.yml").unwrap()).unwrap()
}

/// A root settings struct.
#[derive(Deserialize, Debug)]
pub struct Settings {
    /// Web server port.
    pub http_port: Option<u16>,
    /// Configured services.
    pub services: Vec<ServiceSettings>,
}

/// A service configuration.
#[derive(Deserialize, Debug)]
pub enum ServiceSettings {
    /// Emits an event regularly.
    Clock(services::clock::ClockSettings),
    /// Regularly emits database information.
    Db(services::db::DbSettings),
    /// Dutch Buienradar weather service.
    Buienradar(services::buienradar::BuienradarSettings),
}
