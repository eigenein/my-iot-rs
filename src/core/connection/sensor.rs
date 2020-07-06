#[derive(PartialEq, Debug, Clone)]
pub struct Sensor {
    /// Sensor ID, for example: `buienradar::6240::feel_temperature`.
    pub id: String,

    /// Optional sensor title.
    pub title: Option<String>,

    /// Human-readable location title.
    pub location: String,
}
