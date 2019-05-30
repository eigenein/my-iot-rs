//! Settings.
use serde::Deserialize;
use std::fs::File;
use crate::services::clock::ClockSettings;

/// Read the settings file.
pub fn read() -> Settings {
    serde_yaml::from_reader(File::open("settings.yml").expect("settings.yml")).unwrap_or_default()
}

/// A root settings struct.
#[derive(Deserialize, Debug)]
pub struct Settings {
    /// Configured services.
    #[serde(default)]
    pub services: Vec<ServiceSettings>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { services: vec![] }
    }
}

/// A service configuration.
#[derive(Deserialize, Debug)]
pub enum ServiceSettings {
    /// Emits an event every `interval` seconds.
    Clock(ClockSettings),
}
