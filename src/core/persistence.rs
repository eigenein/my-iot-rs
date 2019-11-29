//! Database interface.

use crate::prelude::*;
use chrono::prelude::*;
use rusqlite::params;
use rusqlite::OptionalExtension;
use std::path::Path;

mod primitives;
pub mod reading;
pub mod sensor;
pub mod thread;
mod value;

// language=sql
const SQL: &str = r#"
    PRAGMA synchronous = NORMAL;
    PRAGMA journal_mode = WAL;
    PRAGMA foreign_keys = ON;

    CREATE TABLE IF NOT EXISTS sensors (
        pk INTEGER PRIMARY KEY NOT NULL,
        sensor_id TEXT UNIQUE NOT NULL,
        timestamp DATETIME NOT NULL
    );

    -- Stores all sensor readings.
    CREATE TABLE IF NOT EXISTS readings (
        sensor_fk INTEGER NOT NULL REFERENCES sensors(pk) ON UPDATE CASCADE ON DELETE CASCADE,
        timestamp DATETIME NOT NULL,
        value BLOB NOT NULL
    );
    -- Descending index on `timestamp` is needed to speed up the select last queries.
    CREATE UNIQUE INDEX IF NOT EXISTS readings_sensor_fk_timestamp ON readings (sensor_fk, timestamp DESC);
"#;

pub fn connect<P: AsRef<Path>>(path: P) -> Result<Connection> {
    let db = Connection::open(path)?;
    db.execute_batch(SQL)?;
    Ok(db)
}

pub fn upsert_reading(db: &Connection, sensor: &Sensor, reading: &Reading) -> Result<()> {
    let timestamp = reading.timestamp.timestamp_millis();
    db.prepare_cached(
        // language=sql
        r#"
            INSERT OR REPLACE INTO sensors (sensor_id, timestamp) VALUES (?1, ?2)
        "#,
    )?
    .execute(params![sensor.sensor_id, timestamp])?;
    db.prepare_cached(
        // language=sql
        r#"
            INSERT OR IGNORE INTO readings (sensor_fk, timestamp, value)
            VALUES ((
                SELECT pk
                FROM sensors
                WHERE sensor_id = ?1
            ), ?2, ?3)
        "#,
    )?
    .execute(params![sensor.sensor_id, timestamp, reading.value.serialize()])?;
    Ok(())
}

pub fn select_actuals(db: &Connection) -> Result<Vec<(Sensor, Reading)>> {
    db.prepare_cached(
        // language=sql
        r#"
            SELECT sensor_id, sensors.timestamp, value
            FROM sensors
            INNER JOIN readings ON readings.timestamp = sensors.timestamp AND readings.sensor_fk = sensors.pk
        "#,
    )?
    .query_map(params![], |row| {
        Ok((
            Sensor {
                sensor_id: row.get_unwrap("sensor_id"),
            },
            Reading {
                timestamp: Local.timestamp_millis(row.get("timestamp")?),
                value: Value::deserialize(row.get("value")?).unwrap(),
            },
        ))
    })?
    .map(|r| r.map_err(Error::from))
    .collect()
}

/// Select database size in bytes.
pub fn select_size(db: &Connection) -> Result<u64> {
    Ok(db
        // language=sql
        .prepare_cached("SELECT page_count * page_size as size FROM pragma_page_count(), pragma_page_size()")?
        .query_row(params![], |row| row.get::<_, i64>(0))
        .map(|v| v as u64)?)
}

/// Select the very last sensor reading.
pub fn select_last_reading(db: &Connection, sensor_id: &str) -> Result<Option<Reading>> {
    Ok(db
        // language=sql
        .prepare_cached(
            r#"
            SELECT readings.timestamp, value
            FROM sensors
            INNER JOIN readings ON readings.timestamp = sensors.timestamp AND readings.sensor_fk = sensors.pk
            WHERE sensors.sensor_id = ?1
            "#,
        )?
        .query_row(params![sensor_id], |row| {
            Ok(Reading {
                timestamp: Local.timestamp_millis(row.get("timestamp")?),
                value: Value::deserialize(row.get("value")?).unwrap(),
            })
        })
        .optional()?)
}

/// Select the latest sensor readings within the given time interval.
#[allow(dead_code)]
pub fn select_readings(db: &Connection, sensor_id: &str, since: &DateTime<Local>) -> Result<Vec<Reading>> {
    db
        // language=sql
        .prepare_cached(
            r#"
            SELECT timestamp, value
            FROM readings
            INNER JOIN sensors ON sensors.pk = readings.sensor_fk
            WHERE sensors.sensor_id = ?1 AND timestamp >= ?2
            ORDER BY timestamp
            "#,
        )?
        .query_map(params![sensor_id, since.timestamp_millis()], |row| {
            Ok(Reading {
                timestamp: Local.timestamp_millis(row.get("timestamp")?),
                value: Value::deserialize(row.get("value")?).unwrap(),
            })
        })?
        .map(|r| r.map_err(Error::from))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    type Result = crate::Result<()>;

    #[test]
    fn double_upsert_keeps_one_record() -> Result {
        let message = Composer::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000))
            .message;

        let db = connect(":memory:")?;
        upsert_reading(&db, &message.sensor, &message.reading)?;
        upsert_reading(&db, &message.sensor, &message.reading)?;

        let reading_count: i64 = db
            // language=sql
            .prepare("SELECT COUNT(*) FROM readings")?
            .query_row(params![], |row| row.get(0))?;
        assert_eq!(reading_count, 1);

        Ok(())
    }

    #[test]
    fn select_last_reading_returns_none_on_empty_database() -> Result {
        let db = connect(":memory:")?;
        assert_eq!(select_last_reading(&db, "test")?, None);
        Ok(())
    }

    #[test]
    fn select_last_reading_ok() -> Result {
        let message = Composer::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000))
            .message;
        let db = connect(":memory:")?;
        upsert_reading(&db, &message.sensor, &message.reading)?;
        assert_eq!(select_last_reading(&db, "test")?, Some(message.reading));
        Ok(())
    }

    #[test]
    fn select_last_reading_returns_newer_reading() -> Result {
        let db = connect(":memory:")?;
        let old = Composer::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_127_000))
            .message;
        upsert_reading(&db, &old.sensor, &old.reading)?;
        let new = Composer::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000))
            .message;
        upsert_reading(&db, &new.sensor, &new.reading)?;
        assert_eq!(select_last_reading(&db, "test")?, Some(new.reading));
        Ok(())
    }

    #[test]
    fn select_actuals_ok() -> Result {
        let message = Composer::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000))
            .compose();
        let db = connect(":memory:")?;
        upsert_reading(&db, &message.sensor, &message.reading)?;
        assert_eq!(select_actuals(&db)?, vec![(message.sensor, message.reading)]);
        Ok(())
    }

    #[test]
    fn existing_sensor_is_reused() -> Result {
        let db = connect(":memory:")?;
        let old = Composer::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000))
            .message;
        upsert_reading(&db, &old.sensor, &old.reading)?;
        let new = Composer::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_129_000))
            .message;
        upsert_reading(&db, &new.sensor, &new.reading)?;

        let reading_count = db
            // language=sql
            .prepare_cached("SELECT COUNT(*) FROM sensors")?
            .query_row(params![], |row| row.get::<_, i64>(0))?;
        assert_eq!(reading_count, 1);

        Ok(())
    }
}
