#[derive(PartialEq, Debug, Clone)]
pub struct Sensor {
    /// Sensor ID, for example: `buienradar::6240::feel_temperature`.
    /// By convention, it must start with a service ID.
    pub id: String,

    /// Optional sensor title.
    pub title: Option<String>,

    /// Human-readable location title.
    pub location: String,

    /// Tells whether the sensor is able to react on `Write` messages.
    /// Still, that has to be implemented in the service itself.
    /// The flag is only a hint for the interfaces.
    pub is_writable: bool,
}
