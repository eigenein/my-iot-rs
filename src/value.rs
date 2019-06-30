use crate::templates;
use rusqlite::types::*;
use serde::{Deserialize, Serialize};

/// Sensor measurement value.
#[derive(Debug, Serialize, Deserialize)]
pub enum Value {
    /// Generic counter.
    Counter(u64),
    /// File size.
    Size(u64),
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
    fn render(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Value::Counter(count) => write!(f, "{}", templates::Counter { count }),
            Value::Size(size) => write!(f, "{}", size), // FIXME
        }
    }
}
