//! Database interface.

use crate::measurement::Measurement;
use crate::value::Value;
use chrono::prelude::*;
use rusqlite::types::*;
use rusqlite::{ToSql, NO_PARAMS};

/// A database connection.
pub struct Db {
    /// Wrapped SQLite connection.
    connection: rusqlite::Connection,
}

/// Create a new database connection.
pub fn new() -> Db {
    let connection = rusqlite::Connection::open("my-iot.sqlite3").unwrap();

    #[rustfmt::skip]
    connection.execute_batch("
        CREATE TABLE IF NOT EXISTS measurements (
            sensor TEXT NOT NULL,
            ts INTEGER NOT NULL,
            value TEXT NOT NULL
        );
        -- Descending index on `ts` is needed to speed up the select latest queries.
        CREATE UNIQUE INDEX IF NOT EXISTS measurements_sensor_ts ON measurements (sensor, ts DESC);
    ").unwrap();

    Db { connection }
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

impl Db {
    /// Insert measurement into database.
    pub fn insert_measurement(&self, measurement: &Measurement) {
        // TODO: `prepare_cached`, see `select_size`.
        #[rustfmt::skip]
        self.connection.execute::<&[&dyn rusqlite::ToSql]>("
            INSERT OR REPLACE INTO measurements (sensor, ts, value)
            VALUES (?1, ?2, ?3)
        ", &[
            &measurement.sensor,
            &measurement.timestamp.timestamp_millis(),
            &measurement.value,
        ]).unwrap();
    }

    /// Select latest measurement for each sensor.
    pub fn select_latest_measurements(&self) -> Vec<Measurement> {
        self.connection
            // TODO: `prepare_cached`, see `select_size`.
            .prepare("SELECT sensor, MAX(ts) as ts, value FROM measurements GROUP BY sensor")
            .unwrap()
            .query_map(NO_PARAMS, |row| {
                Ok(Measurement {
                    sensor: row.get_unwrap("sensor"),
                    timestamp: Local.timestamp_millis(row.get_unwrap("ts")),
                    value: row.get_unwrap("value"),
                })
            })
            .unwrap()
            .map(|result| result.unwrap())
            .collect()
    }

    /// Select database size in bytes.
    pub fn select_size(&self) -> u64 {
        self.connection
            .prepare_cached("SELECT page_count * page_size as size FROM pragma_page_count(), pragma_page_size()")
            .unwrap()
            .query_row(NO_PARAMS, |row| row.get::<_, i64>(0))
            .unwrap() as u64
    }
}
