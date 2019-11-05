//! Database interface.

use crate::prelude::*;
use chrono::prelude::*;
use rusqlite::params;
use std::path::Path;

mod primitives;
pub mod reading;
pub mod sensor;
pub mod thread;
mod value;

const SQL: &str = r#"
    PRAGMA synchronous = NORMAL;
    PRAGMA journal_mode = WAL;
    PRAGMA foreign_keys = ON;

    CREATE TABLE IF NOT EXISTS sensors (
        id INTEGER PRIMARY KEY NOT NULL,
        sensor TEXT UNIQUE NOT NULL,
        last_reading_id INTEGER NULL REFERENCES readings(id) ON UPDATE CASCADE ON DELETE CASCADE
    );

    -- Stores all sensor readings.
    CREATE TABLE IF NOT EXISTS readings (
        id INTEGER PRIMARY KEY NOT NULL,
        sensor_id INTEGER NOT NULL REFERENCES sensors(id) ON UPDATE CASCADE ON DELETE CASCADE,
        timestamp DATETIME NOT NULL,
        value BLOB NOT NULL
    );
    -- Descending index on `timestamp` is needed to speed up the select last queries.
    CREATE UNIQUE INDEX IF NOT EXISTS readings_sensor_id_timestamp ON readings (sensor_id, timestamp DESC);
"#;

pub fn connect<P: AsRef<Path>>(path: P) -> Result<Connection> {
    let db = Connection::open(path)?;
    db.execute_batch(SQL)?;
    Ok(db)
}

pub fn upsert_reading(db: &Connection, sensor: &Sensor, reading: &Reading) -> Result<()> {
    db.prepare_cached("INSERT OR IGNORE INTO sensors (sensor) VALUES (?1)")?
        .execute(params![sensor.sensor])?;
    let timestamp = reading.timestamp.timestamp_millis();
    db.prepare_cached(
        r#"
            INSERT OR IGNORE INTO readings (sensor_id, timestamp, value)
            VALUES ((
                SELECT id
                FROM sensors
                WHERE sensor = ?1
            ), ?2, ?3)
        "#,
    )?
    .execute(params![sensor.sensor, timestamp, reading.value.serialize()])?;
    db.prepare_cached(
        r#"
            UPDATE sensors
            SET last_reading_id = (
                SELECT id
                FROM readings
                WHERE sensor_id = sensors.id AND timestamp = ?2
            )
            WHERE sensor = ?2
        "#,
    )?
    .execute(params![timestamp, sensor.sensor])?;
    Ok(())
}

pub fn select_actuals(db: &Connection) -> Result<Vec<(Sensor, Reading)>> {
    Ok(db
        .prepare_cached(
            r#"
                SELECT sensor, timestamp, value
                FROM sensors
                INNER JOIN readings ON readings.id = sensors.last_reading_id
            "#,
        )?
        .query_map(params![], |row| {
            Ok((
                Sensor {
                    sensor: row.get_unwrap("sensor"),
                },
                Reading {
                    timestamp: Local.from_timestamp_millis(row.get_unwrap("timestamp")),
                    value: Value::deserialize(row.get_unwrap("value"))?,
                },
            ))
        })?
        .collect())
}

/// Select database size in bytes.
pub fn select_size(db: &Connection) -> Result<u64> {
    Ok(db
        .prepare_cached("SELECT page_count * page_size as size FROM pragma_page_count(), pragma_page_size()")?
        .query_row(params![], |row| row.get::<_, i64>(0))
        .map(|v| v as u64)?)
}

/// Select the very last sensor reading.
pub fn select_last_reading(db: &Connection, sensor: &str) -> Result<Option<Reading>> {
    Ok(db
        .prepare_cached(
            r#"
            SELECT timestamp, value
            FROM sensors
            INNER JOIN readings ON readings.id = sensors.last_reading_id
            WHERE sensors.sensor = ?1
            "#,
        )?
        .query_map(params![sensor], |row| {
            Ok(Reading {
                timestamp: Local.from_timestamp_millis(row.get_unwrap("timestamp")),
                value: Value::deserialize(row.get_unwrap("value"))?,
            })
        })?
        .collect())
}

/// Select the latest sensor readings within the given time interval.
pub fn select_readings(db: &Connection, sensor: &str, since: &DateTime<Local>) -> Result<Vec<Reading>> {
    Ok(db
        .prepare_cached(
            r#"
            SELECT timestamp, value
            FROM readings
            INNER JOIN sensors ON sensors.id = readings.sensor_id
            WHERE sensors.sensor = ?1 AND timestamp >= ?2
            ORDER BY timestamp
            "#,
        )?
        .query_map(params![sensor, since.timestamp_millis()], |row| {
            Ok(Reading {
                timestamp: Local.from_timestamp_millis(row.get_unwrap("timestamp")),
                value: Value::deserialize(row.get_unwrap("value"))?,
            })
        })?
        .collect())
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
            .into();

        let db = establish_connection(":memory:")?;
        upsert_reading(&db, &message.sensor, &message.reading)?;
        upsert_reading(&db, &message.sensor, &message.reading)?;

        let reading_count: i64 = db
            .connection
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
            .into();
        let db = connect(":memory:")?;
        upsert_reading(&db, &message.sensor, &message.reading)?;
        assert_eq!(select_last_reading(&db, "test")?, Some(reading));
        Ok(())
    }

    #[test]
    fn select_last_reading_returns_newer_reading() -> Result {
        let db = establish_connection(":memory:")?;
        let old = Composer::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_127_000))
            .into();
        upsert_reading(&db, &old.sensor, &old.reading)?;
        let new = Composer::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000))
            .into();
        upsert_reading(&db, &new.sensor, &new.reading)?;
        assert_eq!(db.select_last_reading("test")?, Some(new));
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
        assert_eq!(select_actuals(&db)?, vec![(&message.sensor, &message.reading)]);
        Ok(())
    }

    #[test]
    fn existing_sensor_is_reused() -> Result {
        let db = connect(":memory:")?;
        let old = &Composer::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000))
            .into();
        upsert_reading(&db, &message.sensor, &message.reading)?;
        let new = Composer::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_129_000))
            .into();
        upsert_reading(&db, &message.sensor, &message.reading)?;

        let reading_count = db
            .prepare_cached("SELECT COUNT(*) FROM sensors")?
            .query_row(params![], |row| row.get::<_, i64>(0))?;
        assert_eq!(reading_count, 1);

        Ok(())
    }
}
