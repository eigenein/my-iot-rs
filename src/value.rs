use rusqlite::types::{ToSql, ToSqlOutput, Value as RusqliteValue};
use rusqlite::Result as RusqliteResult;

/// Sensor measurement value.
#[derive(Debug)]
pub enum Value {
    U64(u64),
}

impl ToSql for Value {
    fn to_sql(&self) -> RusqliteResult<ToSqlOutput> {
        Ok(ToSqlOutput::Owned(RusqliteValue::Blob(self.to_vec())))
    }
}

impl Value {
    pub fn kind(&self) -> u32 {
        match self {
            Value::U64(_) => 1, // TODO: make `Kind` enum.
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        match self {
            Value::U64(value) => value.to_le_bytes().to_vec(),
        }
    }
}
