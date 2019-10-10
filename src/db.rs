//! Database interface.

use crate::message::Reading;
use crate::value::Value;
use crate::Result;
use chrono::prelude::*;
use rusqlite::types::*;
use rusqlite::{params, Connection, Row, ToSql};
use std::path::Path;

const SCHEMA: &str = "
    CREATE TABLE IF NOT EXISTS sensors (
        id INTEGER PRIMARY KEY,
        sensor TEXT UNIQUE NOT NULL,
        last_reading_id INTEGER NULL REFERENCES readings(id) ON UPDATE CASCADE ON DELETE CASCADE
    );

    -- Stores all sensor readings.
    CREATE TABLE IF NOT EXISTS readings (
        id INTEGER PRIMARY KEY,
        sensor_id INTEGER REFERENCES sensors(id) ON UPDATE CASCADE ON DELETE CASCADE,
        ts INTEGER NOT NULL,
        value TEXT NOT NULL
    );
    -- Descending index on `ts` is needed to speed up the select latest queries.
    CREATE UNIQUE INDEX IF NOT EXISTS readings_sensor_id_ts ON readings (sensor_id, ts DESC);

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
        Ok(Db { connection })
    }
}

/// Readings persistence.
impl Db {
    /// Insert reading into database.
    pub fn insert_reading(&self, reading: &Reading) -> Result<()> {
        self.connection
            .prepare_cached("INSERT OR IGNORE INTO sensors (sensor) VALUES (?1)")?
            .execute(params![reading.sensor])?;
        let sensor_id = self.connection.last_insert_rowid();
        self.connection
            .prepare_cached("INSERT OR REPLACE INTO readings (sensor_id, ts, value) VALUES (?1, ?2, ?3)")?
            .execute(params![sensor_id, reading.timestamp.timestamp_millis(), reading.value])?;
        let reading_id = self.connection.last_insert_rowid();
        self.connection
            .prepare_cached("UPDATE sensors SET last_reading_id = ?1 WHERE id = ?2")?
            .execute(params![reading_id, sensor_id])?;
        Ok(())
    }

    /// Select latest reading for each sensor.
    pub fn select_latest_readings(&self) -> Result<Vec<Reading>> {
        self.connection
            .prepare_cached(
                r#"
                SELECT sensors.sensor, MAX(ts) as ts, value
                FROM readings
                INNER JOIN sensors ON sensors.id = readings.sensor_id
                GROUP BY sensors.id
                "#,
            )?
            .query_map(params![], |row| Ok(Reading::from(row)))?
            .map(|result| result.map_err(|e| e.into()))
            .collect()
    }

    /// Select database size in bytes.
    pub fn select_size(&self) -> Result<u64> {
        self.connection
            .prepare_cached("SELECT page_count * page_size as size FROM pragma_page_count(), pragma_page_size()")?
            .query_row(params![], |row| row.get::<_, i64>(0))
            .map(|v| v as u64)
            .map_err(|e| e.into())
    }

    /// Select the very last sensor reading.
    pub fn select_last_reading(&self, sensor: &str) -> Result<Option<Reading>> {
        Ok(self
            .connection
            .prepare_cached(
                r#"
                SELECT sensors.sensor, readings.ts, readings.value
                FROM sensors
                INNER JOIN readings ON readings.id = sensors.last_reading_id
                WHERE sensors.sensor = ?1
                "#,
            )?
            .query_row(params![sensor], |row| Ok(Some(Reading::from(row))))
            .unwrap_or(None))
    }

    /// Select the latest sensor readings within the given time interval.
    pub fn select_readings(&self, sensor: &str, since: &DateTime<Local>) -> Result<Vec<Reading>> {
        Ok(self
            .connection
            .prepare_cached(
                r#"
                SELECT sensors.sensor, ts, value
                FROM readings
                INNER JOIN sensors ON sensors.id = readings.sensor_id
                WHERE sensors.sensor = ?1 AND ts >= ?2
                ORDER BY ts
                "#,
            )?
            .query_map(params![sensor, since.timestamp_millis()], |row| Ok(Reading::from(row)))?
            .map(|result| result.unwrap())
            .collect())
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

#[cfg(test)]
mod tests {
    use super::*;

    type Result = crate::Result<()>;

    #[test]
    fn reading_double_insert_keeps_one_record() -> Result {
        let reading = Reading {
            sensor: "test".into(),
            value: Value::Counter(42),
            timestamp: Local.timestamp_millis(1_566_424_128_000),
        };

        let db = Db::new(":memory:")?;
        db.insert_reading(&reading)?;
        db.insert_reading(&reading)?;

        let reading_count = db
            .connection
            .prepare("SELECT COUNT(*) FROM readings")?
            .query_row(params![], |row| row.get::<_, i64>(0))?;
        assert_eq!(reading_count, 1);

        Ok(())
    }

    #[test]
    fn select_last_reading_returns_none_on_empty_database() -> Result {
        let db = Db::new(":memory:")?;
        assert_eq!(db.select_last_reading("test")?, None);
        Ok(())
    }

    #[test]
    fn select_last_reading_returns_test_reading() -> Result {
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
    fn select_last_reading_returns_newer_reading() -> Result {
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
    fn select_latest_readings_returns_test_reading() -> Result {
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

    #[test]
    fn existing_sensor_is_reused() -> Result {
        let db = Db::new(":memory:")?;
        db.insert_reading(&Reading {
            sensor: "test".into(),
            value: Value::Counter(42),
            timestamp: Local.timestamp_millis(1_566_424_128_000),
        })?;
        db.insert_reading(&Reading {
            sensor: "test".into(),
            value: Value::Counter(42),
            timestamp: Local.timestamp_millis(1_566_424_129_000),
        })?;

        let reading_count = db
            .connection
            .prepare("SELECT COUNT(*) FROM sensors")?
            .query_row(params![], |row| row.get::<_, i64>(0))?;
        assert_eq!(reading_count, 1);

        Ok(())
    }
}
