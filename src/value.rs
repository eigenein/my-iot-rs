use rusqlite::types::{ToSql, ToSqlOutput, Value as RusqliteValue};
use rusqlite::Result as RusqliteResult;
use serde::{Deserialize, Serialize};

/// Sensor measurement value.
#[derive(Debug, Serialize, Deserialize)]
pub enum Value {
    Counter(u64),
}

impl ToSql for Value {
    fn to_sql(&self) -> RusqliteResult<ToSqlOutput> {
        Ok(ToSqlOutput::Owned(RusqliteValue::Text(
            serde_json::to_string(&self).unwrap(),
        )))
    }
}
