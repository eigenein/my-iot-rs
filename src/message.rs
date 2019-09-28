//! Describes a sensor reading and related structures.

use crate::value::Value;
use chrono::prelude::*;
use serde::Deserialize;

/// Services use messages to exchange sensor readings between each other.
#[derive(Debug, Clone)]
pub struct Message {
    pub reading: Reading,

    pub type_: Type,
}

/// Single sensor reading.
///
/// All the sensor data is stored as a single collection of all readings from all services.
#[derive(Debug, PartialEq, Clone)]
pub struct Reading {
    /// A sensor. For example: `buienradar::6240::wind_speed_bft`.
    ///
    /// Note that sensors do not exist as separate entities. Sensor is only a sort of "label"
    /// that corresponds to a sensor in the physical world and used to distinguish readings
    /// between different "real" sensors.
    pub sensor: String,

    /// An attached typed value.
    pub value: Value,

    /// Timestamp when the value was actually measured. This may be earlier than moment of emitting a reading.
    pub timestamp: DateTime<Local>,
}

/// Message type.
#[derive(Clone, Copy, PartialEq, Debug, Deserialize)]
pub enum Type {
    /// Actual sensor reading, which should be persisted in the database and displayed on the dashboard.
    Actual,

    /// Sensor reading which become non-actual just right after it was sent.
    /// Think of, for example, a single camera snapshot.
    OneOff,

    /// Used to control other services. One service may send this to control a sensor of another service.
    Control,
}

impl Message {
    pub fn new<S: Into<String>>(type_: Type, sensor: S, value: Value, timestamp: DateTime<Local>) -> Message {
        Message {
            type_,
            reading: Reading {
                value,
                timestamp,
                sensor: sensor.into(),
            },
        }
    }

    pub fn now<S: Into<String>>(type_: Type, sensor: S, value: Value) -> Message {
        Message::new(type_, sensor, value, Local::now())
    }
}
