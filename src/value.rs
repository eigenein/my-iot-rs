use humansize::FileSize;
use rusqlite::types::*;
use serde::{Deserialize, Serialize};

/// Sensor measurement value.
#[derive(Debug, Serialize, Deserialize)]
pub enum Value {
    /// Generic counter.
    Counter(u64),
    /// File size.
    Size(u64),
    /// Plain text.
    Text(String),
    /// Celsius temperature.
    Celsius(f64),
    /// Beaufort wind speed.
    Bft(u32),
}

impl ToSql for Value {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput> {
        Ok(ToSqlOutput::Owned(rusqlite::types::Value::Text(
            serde_json::to_string(&self).unwrap(),
        )))
    }
}

impl FromSql for Value {
    fn column_result(value: ValueRef) -> Result<Self, FromSqlError> {
        Ok(serde_json::from_str(value.as_str().unwrap()).unwrap())
    }
}

impl markup::Render for Value {
    /// Render value in HTML.
    fn render(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Value::Counter(count) => write!(f, r#"<i class="fas fa-sort-numeric-up-alt"></i> {} times"#, count),
            Value::Size(size) => write!(
                f,
                r#"<i class="far fa-save"></i> {}"#,
                size.file_size(humansize::file_size_opts::DECIMAL).unwrap()
            ),
            Value::Text(ref string) => write!(f, r#"<i class="fas fa-quote-left"></i> {}"#, string),
            Value::Celsius(degrees) => write!(f, r#"<i class="fas fa-thermometer-half"></i> {:.1} ℃"#, degrees),
            Value::Bft(bft) => write!(f, r#"<i class="fas fa-wind"></i> {} BFT"#, bft),
        }
    }
}

impl Value {
    /// Retrieve value color class.
    pub fn class(&self) -> &str {
        match *self {
            Value::Text(_) | Value::Counter(_) | Value::Size(_) => "is-light",
            Value::Bft(_) => "is-light", // TODO
            Value::Celsius(value) => match value {
                // TODO
                _ if 5.0 <= value && value < 15.0 => "is-primary",
                _ if 15.0 <= value && value < 25.0 => "is-success",
                _ => panic!("value {} is not covered", value),
            },
        }
    }
}
