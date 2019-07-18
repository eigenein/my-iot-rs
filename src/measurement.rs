//! Describes a sensor measurement.
use crate::value::Value;
use chrono::prelude::*;

/// A sensor measurement.
#[derive(Debug)]
pub struct Measurement {
    /// A sensor. For example: `buienradar::6240::wind_speed_bft`.
    pub sensor: String,
    /// An attached typed value.
    pub value: Value,
    /// Timestamp when the value was actually measured.
    pub timestamp: DateTime<Local>,
}

impl Measurement {
    /// Create a new measurement.
    pub fn new(sensor: String, value: Value, timestamp: Option<DateTime<Local>>) -> Self {
        Measurement {
            sensor,
            value,
            timestamp: timestamp.unwrap_or_else(Local::now),
        }
    }
}
