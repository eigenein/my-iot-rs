use rusqlite::types::{ToSql, ToSqlOutput, Value as RusqliteValue};
use rusqlite::Result as RusqliteResult;

/// Sensor measurement value.
#[derive(Debug)]
pub enum Value {
    Counter(u64),
}

/// Value kind.
#[derive(Debug)]
pub enum Kind {
    Counter = 1,
}

impl ToSql for Value {
    fn to_sql(&self) -> RusqliteResult<ToSqlOutput> {
        Ok(ToSqlOutput::Owned(RusqliteValue::Blob(self.to_vec())))
    }
}

impl Value {
    pub fn kind(&self) -> Kind {
        match self {
            Value::Counter(_) => Kind::Counter,
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        match self {
            Value::Counter(value) => value.to_le_bytes().to_vec(),
        }
    }
}
