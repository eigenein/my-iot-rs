//! Describes a sensor reading and related structures.

use crate::value::Value;
use chrono::prelude::*;
use serde::Deserialize;

/// Services use messages to exchange sensor readings between each other.
/// Message contains a single sensor reading alongside with some metadata.
#[derive(Debug, Clone)]
pub struct Message {
    /// Message type.
    pub type_: Type,

    /// A sensor. For example: `buienradar::6240::wind_speed_bft`.
    ///
    /// Note that sensors do not exist as separate entities. Sensor is only a sort of "label"
    /// that corresponds to a sensor in the physical world and used to distinguish readings
    /// between different "real" sensors.
    pub sensor: String,

    /// Timestamp when the value was actually measured. This may be earlier than a moment of sending the message.
    pub timestamp: DateTime<Local>,

    /// Attached value.
    pub value: Value,
}

/// Message type.
#[derive(Clone, Copy, PartialEq, Debug, Deserialize)]
pub enum Type {
    /// Normal persistently stored sensor reading. The most frequently used message type.
    ReadLogged,

    /// Sensor reading which become non-actual just right after it was sent, thus not persisted at all.
    /// Think of, for example, a chat message.
    ReadNonLogged,

    /// Sensor reading that invalidates previous reading. Only last reading is stored.
    /// Think of, for example, a camera snapshot.
    ReadSnapshot,

    /// Used to control other services. One service may send this to control a sensor of another service.
    Write,
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

    pub fn now<S: Into<String>, V: Into<Value>>(type_: Type, sensor: S, value: V) -> Message {
        Message::new(type_, sensor, value.into(), Local::now())
    }
}
