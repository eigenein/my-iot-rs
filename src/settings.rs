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
            .map(|path| -> Result<String> {
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
    /// Web server port. It's used for the user interface as well as for webhooks.
    #[serde(default = "default_http_port")]
    pub http_port: u16,

    #[serde(default)]
    pub dashboard: DashboardSettings,

    /// Services configuration.
    /// Each entry is a pair of service ID (defined by user) and service settings.
    /// Service ID is normally used as a sensor prefix, for instance: `service_id::service_sensor`.
    pub services: HashMap<String, Service>,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct DashboardSettings {
    #[serde(default)]
    pub temperature_sensor: Option<String>,

    #[serde(default)]
    pub feel_temperature_sensor: Option<String>,
}

impl Default for DashboardSettings {
    fn default() -> Self {
        Self {
            temperature_sensor: None,
            feel_temperature_sensor: None,
        }
    }
}

/// Service settings section.
#[derive(Deserialize, Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum Service {
    /// Dutch [Buienradar](https://www.buienradar.nl/) weather service.
    Buienradar(services::buienradar::Buienradar),

    /// Regularly emits a counter value.
    Clock(services::clock::Clock),

    /// [Lua](https://www.lua.org/) scripting.
    Lua(services::lua::Lua),

    /// Sunrise and sunset messages.
    Solar(services::solar::Solar),

    /// [tado°](https://www.tado.com/) API.
    Tado(services::tado::Tado),

    /// [Telegram bot](https://core.telegram.org/bots/api) service.
    Telegram(services::telegram::Telegram),

    /// [YouLess](https://www.youless.nl/home.html) kWh meter to ethernet bridge.
    YouLess(services::youless::YouLess),
}

pub fn default_http_port() -> u16 {
    8081
}
