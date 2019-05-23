//! Describes sensor units and corresponding rendering functionality.

/// A sensor unit.
pub enum Unit {
    /// No attached value.
    None,
    /// A plain text.
    Text,
    /// A Celsius temperature.
    Celsius,
}

impl Unit {
    /// Render a value to HTML.
    pub fn html(&self, value: &sqlite::Value) -> String {
        match self {
            Unit::None => String::from(""),
            Unit::Text => String::from(value.as_string().unwrap()),
            Unit::Celsius => format!("{:.2}&nbsp;℃", value.as_float().unwrap()),
        }
    }
}
