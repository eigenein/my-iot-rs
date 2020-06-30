//! Describes a sensor reading and related structures.

use crate::prelude::*;

const DEFAULT_ROOM_TITLE: &str = "Home";

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

impl Message {
    pub fn new<S: Into<String>>(sensor_id: S) -> Self {
        Message {
            type_: Type::ReadLogged,
            sensor: Sensor {
                id: sensor_id.into(),
                title: None,
                room_title: DEFAULT_ROOM_TITLE.into(),
            },
            reading: Reading {
                timestamp: Local::now(),
                value: Value::None,
            },
        }
    }

    pub fn type_(mut self, type_: Type) -> Self {
        self.type_ = type_;
        self
    }

    pub fn value<V: Into<Value>>(mut self, value: V) -> Self {
        self.reading.value = value.into();
        self
    }

    pub fn sensor_title<S: Into<String>>(mut self, sensor_title: S) -> Self {
        self.sensor.title = Some(sensor_title.into());
        self
    }

    pub fn room_title<S: Into<String>>(mut self, room_title: S) -> Self {
        self.sensor.room_title = room_title.into();
        self
    }

    pub fn optional_room_title<S: Into<Option<String>>>(mut self, room_title: S) -> Self {
        self.sensor.room_title = room_title.into().unwrap_or_else(|| DEFAULT_ROOM_TITLE.into());
        self
    }

    pub fn timestamp<T: Into<DateTime<Local>>>(mut self, timestamp: T) -> Self {
        self.reading.timestamp = timestamp.into();
        self
    }
}
