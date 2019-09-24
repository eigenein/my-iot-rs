//! Database interface.

use crate::message::Reading;
use crate::value::Value;
use crate::Result;
use chrono::prelude::*;
use rusqlite::types::*;
use rusqlite::{Connection, Row, ToSql, NO_PARAMS};
use std::path::Path;

const SCHEMA: &str = "
    -- Stores all sensor readings.
    CREATE TABLE IF NOT EXISTS readings (
        sensor TEXT NOT NULL,
        ts INTEGER NOT NULL,
        value TEXT NOT NULL
    );
    -- Descending index on `ts` is needed to speed up the select latest queries.
    CREATE UNIQUE INDEX IF NOT EXISTS readings_sensor_ts ON readings (sensor, ts DESC);

    -- Tables for key-value store.
    CREATE TABLE IF NOT EXISTS integers (
        `key` TEXT NOT NULL PRIMARY KEY,
        value INTEGER,
        expires INTEGER NOT NULL
    );
    CREATE TABLE IF NOT EXISTS reals (
        `key` TEXT NOT NULL PRIMARY KEY,
        value REAL,
        expires INTEGER NOT NULL
    );
    CREATE TABLE IF NOT EXISTS texts (
        `key` TEXT NOT NULL PRIMARY KEY,
        value TEXT,
        expires INTEGER NOT NULL
    );
    CREATE TABLE IF NOT EXISTS blobs (
        `key` TEXT NOT NULL PRIMARY KEY,
        value BLOB,
        expires INTEGER NOT NULL
    );

    -- VACUUM;
";

/// A database connection.
pub struct Db {
    /// Wrapped SQLite connection.
    connection: Connection,
}

impl Db {
    /// Create a new database connection.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Db> {
        let connection = Connection::open(path)?;
        connection.execute_batch(SCHEMA)?;
        let db = Db { connection };
        Ok(db)
    }
}

/// Readings persistence.
impl Db {
    /// Insert reading into database.
    pub fn insert_reading(&self, reading: &Reading) -> Result<()> {
        self.connection
            .prepare_cached("INSERT INTO readings (sensor, ts, value) VALUES (?1, ?2, ?3)")?
            .execute(&[
                &reading.sensor as &dyn ToSql,
                &reading.timestamp.timestamp_millis(),
                &reading.value,
            ])?;
        Ok(())
    }

    /// Select latest reading for each sensor.
    pub fn select_latest_readings(&self) -> Result<Vec<Reading>> {
        self.connection
            .prepare_cached("SELECT sensor, MAX(ts) as ts, value FROM readings GROUP BY sensor")?
            .query_map(NO_PARAMS, |row| Ok(Reading::from(row)))?
            .map(|result| result.map_err(|e| e.into()))
            .collect()
    }

    /// Select database size in bytes.
    pub fn select_size(&self) -> Result<u64> {
        self.connection
            .prepare_cached("SELECT page_count * page_size as size FROM pragma_page_count(), pragma_page_size()")?
            .query_row(NO_PARAMS, |row| row.get::<_, i64>(0))
            .map(|v| v as u64)
            .map_err(|e| e.into())
    }

    /// Select the very last sensor reading.
    pub fn select_last_reading(&self, sensor: &str) -> Result<Option<Reading>> {
        Ok(self
            .connection
            .prepare_cached("SELECT sensor, ts, value FROM readings WHERE sensor = ?1 ORDER BY ts DESC LIMIT 1")?
            .query_row(&[&sensor as &dyn ToSql], |row| Ok(Some(Reading::from(row))))
            .unwrap_or(None))
    }

    /// Select the latest sensor readings within the given time interval.
    pub fn select_readings(&self, sensor: &str, since: &DateTime<Local>) -> Result<Vec<Reading>> {
        Ok(self
            .connection
            .prepare_cached("SELECT sensor, ts, value FROM readings WHERE sensor = ?1 AND ts >= ?2 ORDER BY ts")?
            .query_map(&[&sensor as &dyn ToSql, &since.timestamp_millis()], |row| {
                Ok(Reading::from(row))
            })?
            .map(|result| result.unwrap())
            .collect())
    }
}

/// Key-value store.
impl Db {
    /// Get an item from the key-value store.
    pub fn get<K, V>(&self, key: K) -> Result<Option<V>>
    where
        K: AsRef<str>,
        V: SqliteTypeName + FromSql,
    {
        Ok(self
            .connection
            .prepare_cached(&format!(
                "SELECT value FROM {}s WHERE `key` = ?1 AND expires > ?2",
                V::name()
            ))?
            .query_row(
                &[&key.as_ref() as &dyn ToSql, &Local::now().timestamp_millis()],
                |row| Ok(Some(row.get_unwrap::<_, V>("value"))),
            )
            .unwrap_or(None))
    }

    /// Set item in generic key-value store.
    pub fn set<K, V, E>(&self, key: K, value: V, expires_at: E) -> Result<()>
    where
        K: AsRef<str>,
        V: SqliteTypeName + ToSql,
        E: Into<DateTime<Local>>,
    {
        self.connection
            .prepare_cached(&format!(
                "INSERT OR REPLACE INTO {}s (`key`, value, expires) VALUES (?1, ?2, ?3)",
                V::name()
            ))?
            .execute(&[
                &key.as_ref() as &dyn ToSql,
                &value,
                &expires_at.into().timestamp_millis(),
            ])?;
        Ok(())
    }
}

/// Serializes value to JSON.
impl ToSql for Value {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput> {
        match serde_json::to_string(&self) {
            Ok(string) => Ok(ToSqlOutput::Owned(rusqlite::types::Value::Text(string))),
            Err(error) => Err(rusqlite::Error::ToSqlConversionFailure(Box::new(error))),
        }
    }
}

/// De-serializes value from JSON.
impl FromSql for Value {
    fn column_result(value: ValueRef) -> FromSqlResult<Self> {
        match serde_json::from_str(value.as_str()?) {
            Ok(value) => Ok(value),
            Err(error) => Err(FromSqlError::Other(Box::new(error))),
        }
    }
}

/// Initializes reading from database row.
impl From<&Row<'_>> for Reading {
    fn from(row: &Row<'_>) -> Self {
        Reading {
            sensor: row.get_unwrap("sensor"),
            timestamp: Local.timestamp_millis(row.get_unwrap("ts")),
            value: row.get_unwrap("value"),
        }
    }
}

/// Trait which returns SQLite type name of the implementing type.
pub trait SqliteTypeName {
    fn name() -> &'static str;
}

impl SqliteTypeName for i32 {
    fn name() -> &'static str {
        "integer"
    }
}

impl SqliteTypeName for f64 {
    fn name() -> &'static str {
        "real"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    type Result = crate::Result<()>;

    #[test]
    fn set_and_get() -> Result {
        let db = Db::new(":memory:")?;
        db.set("hello", 42, Local::now() + Duration::days(1))?;
        assert_eq!(db.get::<_, i32>("hello").unwrap(), Some(42));
        Ok(())
    }

    #[test]
    fn get_missing_returns_none() -> Result {
        assert_eq!(Db::new(":memory:")?.get::<_, i32>("missing")?, None);
        Ok(())
    }

    #[test]
    fn expired_returns_none() -> Result {
        let db = Db::new(":memory:")?;
        db.set("hello", 42, Local::now())?;
        assert_eq!(db.get::<_, i32>("hello")?, None);
        Ok(())
    }

    #[test]
    fn cannot_get_different_type_value() -> Result {
        let db = Db::new(":memory:")?;
        db.set("hello", 42, Local::now() + Duration::days(1))?;
        assert_eq!(db.get::<_, f64>("hello")?, None);
        Ok(())
    }

    #[test]
    fn select_last_reading() -> Result {
        let reading = Reading {
            sensor: "test".into(),
            value: Value::Counter(42),
            timestamp: Local.timestamp_millis(1_566_424_128_000),
        };
        let db = Db::new(":memory:")?;
        db.insert_reading(&reading)?;
        assert_eq!(db.select_last_reading("test")?, Some(reading));
        Ok(())
    }

    #[test]
    fn overwrite_and_select_last_reading() -> Result {
        let db = Db::new(":memory:")?;
        db.insert_reading(&Reading {
            sensor: "test".into(),
            value: Value::Counter(42),
            timestamp: Local.timestamp_millis(1_566_424_127_000),
        })?;
        let new = Reading {
            sensor: "test".into(),
            value: Value::Counter(42),
            timestamp: Local.timestamp_millis(1_566_424_128_000),
        };
        db.insert_reading(&new)?;
        assert_eq!(db.select_last_reading("test")?, Some(new));
        Ok(())
    }

    #[test]
    fn select_latest_readings() -> Result {
        let reading = Reading {
            sensor: "test".into(),
            value: Value::Counter(42),
            timestamp: Local.timestamp_millis(1_566_424_128_000),
        };
        let db = Db::new(":memory:")?;
        db.insert_reading(&reading)?;
        assert_eq!(db.select_latest_readings()?, vec![reading]);
        Ok(())
    }
}
