//! Describes a sensor event.

use sqlite::Value;

/// A sensor event.
#[derive(Debug)]
pub struct Event {
    /// A sensor. For example: `buienradar::6240::wind_speed_bft`.
    pub sensor: String,
    /// An attached typed value.
    pub value: Value,
    // TODO: timestamp
}
