//! Diesel ORM models.

use crate::prelude::*;
use chrono::{DateTime, Local};
use diesel::{Identifiable, Insertable, Queryable};

#[derive(Queryable, Insertable, Identifiable, PartialEq)]
pub struct Sensor {
    pub id: u64,

    /// A sensor. For example: `buienradar::6240::wind_speed_bft`.
    ///
    /// Note that sensors do not exist as separate entities. Sensor is only a sort of "label"
    /// that corresponds to a sensor in the physical world and used to distinguish readings
    /// between different "real" sensors.
    pub sensor: String,

    pub last_reading_id: Option<u64>,
}

#[derive(Queryable, Insertable, Identifiable, Associations, PartialEq)]
#[belongs_to(Sensor)]
pub struct Reading {
    pub id: u64,
    pub sensor_id: u64,

    /// Timestamp when the value was actually measured. This may be earlier than a moment of sending the message.
    pub timestamp: DateTime<Local>,

    /// Attached value.
    pub value: Value,
}
