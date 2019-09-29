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
use crate::{threading, Result};
use crossbeam_channel::Sender;
use log::{debug, info};
use serde::Deserialize;
use std::sync::{Arc, Mutex};

/// Automator settings.
#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    scenarios: Vec<Scenario>,
}

pub fn spawn(
    service_id: &str,
    settings: &Settings,
    db: &Arc<Mutex<Db>>,
    tx: &Sender<Message>,
) -> Result<Vec<Sender<Message>>> {
    let tx = tx.clone();
    let db = db.clone();
    let service_id = service_id.to_string();
    let settings = settings.clone();

    let (out_tx, rx) = crossbeam_channel::unbounded::<Message>();

    threading::spawn(format!("my-iot::automator:{}", &service_id), move || {
        for message in rx {
            for scenario in settings.scenarios.iter() {
                if scenario.conditions.iter().all(|c| c.is_met(&message.reading)) {
                    info!(
                        r"{} triggered scenario: {}",
                        &message.reading.sensor, scenario.description
                    );
                    for action in scenario.actions.iter() {
                        action.execute(&service_id, &db, &message.reading, &tx).unwrap();
                    }
                } else {
                    debug!("Skipped: {}", &message.reading.sensor);
                }
            }
        }
        unreachable!();
    })?;

    Ok(vec![out_tx])
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

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "action")]
pub enum Action {
    /// Emit a message with the original value and custom message type and sensor.
    Repeat(RepeatParameters),

    /// Read the last sensor value and emit a message with the same value but custom sensor and type.
    /// If the former is missing, then no message will be sent.
    ReadSensor(ReadSensorParameters),
}

#[derive(Deserialize, Debug, Clone)]
pub struct RepeatParameters {
    target_type: Type,
    target_sensor: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ReadSensorParameters {
    source_sensor: String,
    target_type: Type,
    target_sensor: String,
}

impl Action {
    pub fn execute(
        &self,
        _service_id: &str,
        db: &Arc<Mutex<Db>>,
        reading: &Reading,
        tx: &Sender<Message>,
    ) -> Result<()> {
        match self {
            Action::Repeat(parameters) => {
                tx.send(Message::now(
                    parameters.target_type,
                    &parameters.target_sensor,
                    reading.value.clone(),
                ))?;
                Ok(())
            }
            Action::ReadSensor(parameters) => {
                if let Some(source) = db.lock().unwrap().select_last_reading(&parameters.source_sensor)? {
                    tx.send(Message::now(
                        parameters.target_type,
                        &parameters.target_sensor,
                        source.value,
                    ))?
                }
                Ok(())
            }
        }
    }
}
