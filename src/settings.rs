//! Settings.
use crate::services::clock::ClockSettings;
use serde::Deserialize;
use std::fs::File;

/// Read the settings file.
pub fn read() -> Settings {
    serde_yaml::from_reader(File::open("settings.yml").unwrap()).unwrap()
}

/// A root settings struct.
#[derive(Deserialize, Debug)]
pub struct Settings {
    /// Configured services.
    pub services: Vec<ServiceSettings>,
}

/// A service configuration.
#[derive(Deserialize, Debug)]
pub enum ServiceSettings {
    /// Emits an event every `interval` seconds.
    Clock(ClockSettings),
}
