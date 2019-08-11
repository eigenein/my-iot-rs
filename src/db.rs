//! Database interface.

use crate::reading::Reading;
use crate::value::Value;
use chrono::prelude::*;
use rusqlite::types::*;
use rusqlite::{Connection, Row, ToSql, NO_PARAMS};
use std::path::Path;

/// A database connection.
pub struct Db {
    /// Wrapped SQLite connection.
    connection: Connection,
}

impl Db {
    /// Create a new database connection.
    pub fn new<P: AsRef<Path>>(path: P) -> Db {
        let connection = Connection::open(path).unwrap();

        #[rustfmt::skip]
        connection.execute_batch("
            -- Stores all sensor readings.
            CREATE TABLE IF NOT EXISTS readings (
                sensor TEXT NOT NULL,
                ts INTEGER NOT NULL,
                value TEXT NOT NULL
            );
            -- Descending index on `ts` is needed to speed up the select latest queries.
            CREATE UNIQUE INDEX IF NOT EXISTS readings_sensor_ts ON readings (sensor, ts DESC);

            -- Key-value store for general use.
            CREATE TABLE IF NOT EXISTS kv (
                `key` TEXT NOT NULL PRIMARY KEY,
                value TEXT NOT NULL,
                expires_ts INTEGER NOT NULL
            );
        ").unwrap();

        Db { connection }
    }
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

impl From<&Row<'_>> for Reading {
    fn from(row: &Row<'_>) -> Self {
        Reading {
            sensor: row.get_unwrap("sensor"),
            timestamp: Local.timestamp_millis(row.get_unwrap("ts")),
            value: row.get_unwrap("value"),
        }
    }
}

impl Db {
    /// Insert reading into database.
    pub fn insert_reading(&self, reading: &Reading) {
        #[rustfmt::skip]
        self.connection
            .prepare_cached("INSERT OR REPLACE INTO readings (sensor, ts, value) VALUES (?1, ?2, ?3)")
            .unwrap()
            .execute(&[
                &reading.sensor as &dyn ToSql,
                &reading.timestamp.timestamp_millis(),
                &reading.value,
            ])
            .unwrap();
    }

    /// Select latest reading for each sensor.
    pub fn select_latest_readings(&self) -> Vec<Reading> {
        self.connection
            .prepare_cached("SELECT sensor, MAX(ts) as ts, value FROM readings GROUP BY sensor")
            .unwrap()
            .query_map(NO_PARAMS, |row| Ok(Reading::from(row)))
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

    /// Select latest readings for an individual sensor.
    pub fn select_sensor_readings(&self, sensor: &str, since: &DateTime<Local>) -> (Reading, Vec<Reading>) {
        let last = self
            .connection
            .prepare_cached("SELECT sensor, ts, value FROM readings WHERE sensor = ?1 ORDER BY ts DESC LIMIT 1")
            .unwrap()
            .query_row(&[&sensor as &dyn ToSql], |row| Ok(Reading::from(row)))
            .unwrap();
        let readings = self
            .connection
            .prepare_cached("SELECT sensor, ts, value FROM readings WHERE sensor = ?1 AND ts >= ?2 ORDER BY ts")
            .unwrap()
            .query_map(&[&sensor as &dyn ToSql, &since.timestamp_millis()], |row| {
                Ok(Reading::from(row))
            })
            .unwrap()
            .map(|result| result.unwrap())
            .collect();
        (last, readings)
    }

    /// Get item from generic key-value store.
    pub fn get<K: AsRef<str>>(&self, key: K) -> serde_json::Value {
        self.connection
            .prepare_cached("SELECT value FROM kv WHERE `key` = ?1 AND expires_ts > ?2")
            .unwrap()
            .query_row(
                &[&key.as_ref() as &dyn ToSql, &Local::now().timestamp_millis()],
                |row| Ok(serde_json::from_str(&row.get_unwrap::<_, String>("value")).unwrap()),
            )
            .unwrap_or(serde_json::Value::Null)
    }

    /// Set item in generic key-value store.
    pub fn set<K, V, E>(&self, key: K, value: V, expires_at: E)
    where
        K: AsRef<str>,
        V: Into<serde_json::Value>,
        E: Into<DateTime<Local>>,
    {
        self.connection
            .prepare_cached("INSERT OR REPLACE INTO kv (`key`, value, expires_ts) VALUES (?1, ?2, ?3)")
            .unwrap()
            .execute(&[
                &key.as_ref() as &dyn ToSql,
                &serde_json::to_string(&value.into()).unwrap(),
                &expires_at.into().timestamp_millis(),
            ])
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn get_returns_set_value() {
        let db = Db::new(":memory:");
        db.set("hello", "world", Local::now() + Duration::days(1));
        assert_eq!(db.get("hello"), "world");
    }

    #[test]
    fn missing_index_returns_null() {
        assert_eq!(Db::new(":memory:").get("non-existing"), serde_json::Value::Null);
    }

    #[test]
    fn expired_key_returns_null() {
        let db = Db::new(":memory:");
        db.set("hello", "world", Local::now());
        assert_eq!(db.get("hello"), serde_json::Value::Null);
    }
}
