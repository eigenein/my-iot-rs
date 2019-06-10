//! Describes sensor values and corresponding rendering functionality.
use rusqlite::types::{ToSqlOutput, Value as RusqliteValue};
use serde::{Deserialize, Serialize};

/// A sensor value.
#[derive(Debug, Serialize, Deserialize)]
pub enum Value {
    /// No attached value.
    None,
    /// Generic counter.
    Count(u64),
    /// A plain text.
    Text(String),
    /// A Celsius temperature.
    Celsius(f64),
}

impl Value {
    /// Render a value to HTML.
    pub fn html(&self) -> String {
        match self {
            Value::None => String::from(""),
            Value::Text(text) => text.clone(),
            Value::Celsius(degrees) => format!("{:.2}&nbsp;℃", degrees),
            Value::Count(counter) => format!("{}", counter),
        }
    }
}

impl rusqlite::ToSql for Value {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput> {
        let buf = rmp_serde::to_vec(&self).unwrap();
        Ok(ToSqlOutput::Owned(RusqliteValue::Blob(buf)))
    }
}
