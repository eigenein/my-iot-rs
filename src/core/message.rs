//! Describes a sensor reading and related structures.

use crate::core::value::Value;
use chrono::prelude::*;
use serde::Deserialize;

// TODO: still make a separate `Reading` struct which only used to store readings in database.
/// Services use messages to exchange sensor readings between each other.
/// Message contains a single sensor reading alongside with some metadata.
#[derive(Debug, Clone, PartialEq)]
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

/// Message builder. Prefer to use it instead of directly instantiating a `Message`.
pub struct Composer {
    message: Message,
}

impl Composer {
    pub fn new<S: Into<String>>(sensor: S) -> Self {
        Self {
            message: Message {
                sensor: sensor.into(),
                type_: Type::ReadLogged,
                timestamp: Local::now(),
                value: Value::None,
            },
        }
    }

    pub fn type_(mut self, type_: Type) -> Self {
        self.message.type_ = type_;
        self
    }

    pub fn timestamp<T: Into<DateTime<Local>>>(mut self, timestamp: T) -> Self {
        self.message.timestamp = timestamp.into();
        self
    }

    pub fn value<V: Into<Value>>(mut self, value: V) -> Self {
        self.message.value = value.into();
        self
    }
}

impl From<Composer> for Message {
    fn from(composer: Composer) -> Self {
        composer.message
    }
}
