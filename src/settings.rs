//! Settings.
use serde::Deserialize;
use std::fs::File;

/// Read the settings file.
pub fn read() -> Settings {
    serde_yaml::from_reader(File::open("settings.yml").expect("settings.yml")).unwrap_or_default()
}

/// A root settings struct.
#[derive(Deserialize, Debug)]
pub struct Settings {
    /// Configured services.
    #[serde(default)]
    pub services: Vec<Service>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { services: vec![] }
    }
}

/// A service configuration.
#[derive(Deserialize, Debug)]
pub enum Service {
    /// Emits an event every `interval` seconds.
    Clock(ClockSettings),
}

/// Clock settings.
#[derive(Deserialize, Debug)]
pub struct ClockSettings {
    /// Interval in seconds.
    #[serde(default)]
    pub interval: f64,
}

impl Default for ClockSettings {
    fn default() -> Self {
        ClockSettings { interval: 1.0 }
    }
}
