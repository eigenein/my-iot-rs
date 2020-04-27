//! Describes a sensor reading and related structures.

use crate::prelude::*;
use chrono::prelude::*;

/// Services use messages to exchange sensor readings between each other.
/// Message contains a single sensor reading alongside with some metadata.
#[derive(Debug, Clone)]
pub struct Message {
    /// Message type.
    pub type_: Type,

    /// Associated sensor instance.
    pub sensor: Sensor,

    /// Associated sensor reading.
    pub reading: Reading,

    pub metadata: Metadata,
}

/// Message type.
#[derive(Clone, Copy, PartialEq, Debug, Deserialize)]
pub enum Type {
    /// Normal persistently stored sensor reading. The most frequently used message type.
    ReadLogged,

    /// Sensor reading which become non-actual just right after it was sent, thus not persisted at all.
    /// Think of, for example, a chat message.
    ReadNonLogged,

    /// Sensor reading that invalidates previous reading. Only last reading gets stored.
    /// Think of, for example, a camera snapshot.
    ReadSnapshot,

    /// Used to control other services. One service may send this to control a sensor of another service.
    Write,
}

/// Unfortunately, not everything nicely falls into the model of sensors and readings.
/// Thus, each message may contain additional metadata or flags that are very service-specific.
/// Try to use them as little as possible.
#[derive(Debug, Clone)]
pub struct Metadata {
    /// Useful for (for example) instant messaging, where notifications may be fine-tunable.
    pub enable_notification: Option<bool>,
}

impl Default for Metadata {
    fn default() -> Self {
        Metadata {
            enable_notification: None,
        }
    }
}

/// Message builder. Prefer to use it instead of directly instantiating a `Message`.
pub struct Composer {
    pub message: Message,
}

impl Composer {
    pub fn new<S: Into<String>>(sensor_id: S) -> Self {
        Self {
            message: Message {
                type_: Type::ReadLogged,
                sensor: Sensor::new(sensor_id),
                reading: Reading::new(),
                metadata: Metadata::default(),
            },
        }
    }

    pub fn type_(mut self, type_: Type) -> Self {
        self.message.type_ = type_;
        self
    }

    pub fn timestamp<T: Into<DateTime<Local>>>(mut self, timestamp: T) -> Self {
        self.message.reading.timestamp = timestamp.into();
        self
    }

    pub fn value<V: Into<Value>>(mut self, value: V) -> Self {
        self.message.reading.value = value.into();
        self
    }

    pub fn title<T: Into<String>>(mut self, title: T) -> Self {
        self.message.sensor.title = Some(title.into());
        self
    }

    pub fn room_title<T: Into<String>>(mut self, room_title: T) -> Self {
        self.message.sensor.room_title = Some(room_title.into());
        self
    }

    #[allow(dead_code)]
    pub fn compose(self) -> Message {
        self.message
    }
}

impl From<Composer> for Message {
    fn from(composer: Composer) -> Self {
        composer.message
    }
}
