//! Describes a sensor reading.
//!
//! This is a key data structure. All the data is stored as a single list of all readings from all services.

use crate::value::Value;
use chrono::prelude::*;

/// A sensor reading.
#[derive(Debug)]
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

    /// Should the reading be persisted in the database.
    pub is_persisted: bool,
}
