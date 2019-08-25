//! Automation service.
//!
//! Automation _is not_ a special core functionality. Instead, it's implemented as a service,
//! that listens to other services readings and reacts by emitting its own readings.
//!
//! The latter ones, automator-generated readings, are treated in the same way, allowing those to be
//! displayed on the dashboard or caught by other services.
//!
//! Basically, this is a case of "multi-producer multi-consumer" pattern.

use crate::db::Db;
use crate::reading::Reading;
use crate::services::Service;
use crate::{threading, Result};
use chrono::Local;
use log::{debug, info};
use multiqueue::{BroadcastReceiver, BroadcastSender};
use serde::Deserialize;
use std::sync::{Arc, Mutex};

/// Automator settings.
///
/// # Example
///
/// ```yaml
/// services:
///   heartbeat:
///     Clock:
///       interval_ms: 2000
///   automator:
///     Automator:
///       scenarios:
///         - description: re-emit heartbeat readings
///           conditions:
///             - Sensor: heartbeat
///           actions:
///             - Reading: []
/// ```
#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    scenarios: Vec<Scenario>,
}

/// Single automation scenario.
///
/// # Example
///
/// ```yaml
/// scenarios:
///   - description: Re-emit heartbeat readings
///     conditions:
///       - Sensor: heartbeat
///     actions:
///       - Reading: []
/// ```
#[derive(Deserialize, Debug, Clone)]
pub struct Scenario {
    /// User-defined description. Brings no functional effect, but helps to debug scenarios.
    #[serde(default = "String::new")]
    description: String,

    /// Conditions which trigger a scenario to run. All of them must be met in order to trigger
    /// the scenario.
    conditions: Vec<Condition>,

    /// Actions executed when scenario is run.
    actions: Vec<Action>,
}

#[derive(Deserialize, Debug, Clone)]
pub enum Condition {
    /// Sensor matches a specified string.
    ///
    /// # Example
    ///
    /// ```yaml
    /// conditions:
    //    - Sensor: heartbeat
    /// ```
    Sensor(String),

    /// Sensor ends with a specified string.
    SensorEndsWith(String),

    /// Sensor starts with a specified string.
    SensorStartsWith(String),

    /// Sensor contains a specified string.
    SensorContains(String),

    /// At least one of conditions is met.
    Or(Box<Condition>, Box<Condition>),
}

#[derive(Deserialize, Debug, Clone)]
pub enum Action {
    /// Emit a simple reading with original reading value and sensor concatenated from the automator
    /// service ID and original sensor.
    Reading(),
}

/// Automation service.
pub struct Automator {
    service_id: String,
    settings: Settings,
}

impl Automator {
    pub fn new(service_id: &str, settings: &Settings) -> Automator {
        Automator {
            service_id: service_id.into(),
            settings: settings.clone(),
        }
    }
}

impl Service for Automator {
    fn spawn(
        self: Box<Self>,
        _db: Arc<Mutex<Db>>,
        tx: &BroadcastSender<Reading>,
        rx: &BroadcastReceiver<Reading>,
    ) -> Result<()> {
        let tx = tx.clone();
        let rx = rx.add_stream().into_single().unwrap();

        threading::spawn(self.service_id.clone(), move || {
            for reading in rx {
                for scenario in self.settings.scenarios.iter() {
                    if scenario.conditions.iter().all(|c| c.is_met(&reading)) {
                        info!(r#"{} triggered scenario: "{}"."#, &reading.sensor, scenario.description);
                        for action in scenario.actions.iter() {
                            action.execute(&self.service_id, &reading, &tx).unwrap();
                        }
                    } else {
                        debug!("Skipped: {}.", &reading.sensor);
                    }
                }
            }
            unreachable!();
        })?;

        Ok(())
    }
}

impl Condition {
    pub fn is_met(&self, reading: &Reading) -> bool {
        match self {
            Condition::Sensor(sensor) => &reading.sensor == sensor,
            Condition::SensorEndsWith(suffix) => reading.sensor.ends_with(suffix),
            Condition::SensorStartsWith(prefix) => reading.sensor.starts_with(prefix),
            Condition::SensorContains(infix) => reading.sensor.contains(infix),
            Condition::Or(condition_1, condition_2) => condition_1.is_met(&reading) || condition_2.is_met(&reading),
        }
    }
}

impl Action {
    pub fn execute(&self, service_id: &str, reading: &Reading, tx: &BroadcastSender<Reading>) -> Result<()> {
        match self {
            Action::Reading() => tx
                .try_send(Reading {
                    sensor: format!("{}::{}", &service_id, &reading.sensor),
                    timestamp: Local::now(),
                    value: reading.value.clone(),
                    is_persisted: false,
                })
                .map_err(|e| e.into()),
        }
    }
}
