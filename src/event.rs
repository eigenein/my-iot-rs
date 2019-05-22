/// Values that can be attached to an event.
#[derive(Debug)]
pub enum Value {
    /// No attached value.
    None,
    /// Plain text.
    Text(String),
    /// Celsius temperature.
    Celsius(f64),
    /// Wind speed.
    Beaufort(f64),
    /// Boolean.
    Boolean(bool),
    // TODO: DateTime
    // TODO: HPa
    // TODO: ImageUrl
    // TODO: Jpeg
    // TODO: m/s
    // TODO: Rh
    // TODO: time delta
    // TODO: Watt
}

/// Sensor event.
#[derive(Debug)]
pub struct Event {
    /// Unique channel identifier. For example: `buienradar::6240::wind_speed_bft`.
    pub channel: String,
    /// Attached value.
    pub value: Value,
    // TODO: timestamp
}
