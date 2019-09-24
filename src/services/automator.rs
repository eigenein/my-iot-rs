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
use crate::message::{Message, Reading, Type};
use crate::services::Service;
use crate::{threading, Result};
use bus::Bus;
use chrono::Local;
use crossbeam_channel::Sender;
use log::{debug, info};
use serde::Deserialize;
use std::sync::{Arc, Mutex};

/// Automator settings.
#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    scenarios: Vec<Scenario>,
}

/// Single automation scenario.
#[derive(Deserialize, Debug, Clone)]
pub struct Scenario {
    /// User-defined description. Brings no functional effect, but helps to debug scenarios.
    #[serde(default = "String::new")]
    description: String,

    /// Conditions which trigger a scenario to run. All of them must be met in order to trigger
    /// the scenario.
    #[serde(default = "Vec::new")]
    conditions: Vec<Condition>,

    /// Actions executed when scenario is run.
    #[serde(default = "Vec::new")]
    actions: Vec<Action>,
}

#[derive(Deserialize, Debug, Clone)]
pub enum Condition {
    /// Sensor matches a specified string.
    Sensor(String),

    /// Sensor ends with a specified string.
    SensorEndsWith(String),

    /// Sensor starts with a specified string.
    SensorStartsWith(String),

    /// Sensor contains a specified string.
    SensorContains(String),

    /// At least one of conditions is met.
    Or(Vec<Condition>),
}

#[derive(Deserialize, Debug, Clone)]
pub enum Action {
    /// Emit a message with the original value and custom message type and sensor.
    Repeat(Type, String),
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
    fn spawn(self: Box<Self>, _db: Arc<Mutex<Db>>, tx: &Sender<Message>, bus: &mut Bus<Message>) -> Result<()> {
        let tx = tx.clone();
        let rx = bus.add_rx();

        threading::spawn(format!("my-iot::automator:{}", &self.service_id), move || {
            for message in rx {
                for scenario in self.settings.scenarios.iter() {
                    if scenario.conditions.iter().all(|c| c.is_met(&message.reading)) {
                        info!(
                            r#"{} triggered scenario: "{}"."#,
                            &message.reading.sensor, scenario.description
                        );
                        for action in scenario.actions.iter() {
                            action.execute(&self.service_id, &message.reading, &tx).unwrap();
                        }
                    } else {
                        debug!("Skipped: {}.", &message.reading.sensor);
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
            Condition::Or(conditions) => conditions.iter().any(|c| c.is_met(&reading)),
        }
    }
}

impl Action {
    pub fn execute(&self, _service_id: &str, reading: &Reading, tx: &Sender<Message>) -> Result<()> {
        match self {
            Action::Repeat(type_, sensor) => tx
                .try_send(Message {
                    type_: *type_,
                    reading: Reading {
                        sensor: sensor.into(),
                        timestamp: Local::now(),
                        value: reading.value.clone(),
                    },
                })
                .map_err(|e| e.into()),
        }
    }
}
