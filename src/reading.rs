//! Describes a sensor reading.
use crate::value::Value;
use chrono::prelude::*;

/// A sensor reading.
#[derive(Debug)]
pub struct Reading {
    /// A sensor. For example: `buienradar::6240::wind_speed_bft`.
    pub sensor: String,
    /// An attached typed value.
    pub value: Value,
    /// Timestamp when the value was actually measured.
    pub timestamp: DateTime<Local>,
}

impl Reading {
    /// Create a new reading.
    pub fn new(sensor: String, value: Value, timestamp: Option<DateTime<Local>>) -> Self {
        Reading {
            sensor,
            value,
            timestamp: timestamp.unwrap_or_else(Local::now),
        }
    }
}
