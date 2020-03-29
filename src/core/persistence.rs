//! Database interface.

use crate::prelude::*;
use chrono::prelude::*;
use rusqlite::OptionalExtension;
use rusqlite::{params, Row};
use std::path::Path;

mod migrations;
mod primitives;
pub mod reading;
pub mod sensor;
pub mod thread;

#[derive(Debug, PartialEq)]
pub struct Actual {
    pub sensor: Sensor,
    pub reading: Reading,
}

pub fn connect<P: AsRef<Path>>(path: P) -> Result<Connection> {
    let mut db = Connection::open(path)?;
    // language=sql
    db.execute_batch("PRAGMA foreign_keys = ON;")?;

    let version = get_version(&db)? as usize;
    for (i, migrate) in migrations::MIGRATIONS.iter().enumerate() {
        if version < i {
            info!("Migrating to version {}…", i);
            let tx = db.transaction()?;
            migrate(&tx)?;
            tx.pragma_update(None, "user_version", &(i as i32))?;
            tx.commit()?;
        }
    }

    Ok(db)
}

/// Get the database `user_version`.
pub fn get_version(db: &Connection) -> Result<i32> {
    let version: i32 = db.pragma_query_value(None, "user_version", |row| row.get(0))?;
    Ok(version)
}

pub fn upsert_sensor(db: &Connection, sensor: &Sensor, timestamp: i64) -> Result<()> {
    db.prepare_cached(
        // language=sql
        r#"
            -- noinspection SqlResolve @ any/"excluded"
            -- noinspection SqlInsertValues
            INSERT INTO sensors (sensor_id, title, timestamp, room_title) VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT (sensor_id) DO UPDATE SET
                timestamp = excluded.timestamp,
                title = excluded.title,
                room_title = excluded.room_title
        "#,
    )?
    .execute(params![sensor.sensor_id, sensor.title, timestamp, sensor.room_title])?;
    Ok(())
}

pub fn upsert_reading(db: &Connection, sensor: &Sensor, reading: &Reading) -> Result<()> {
    let timestamp = reading.timestamp.timestamp_millis();
    upsert_sensor(&db, &sensor, timestamp)?;

    let mut serialized_value = Vec::new();
    reading.value.serialize(
        &mut rmp_serde::Serializer::new(&mut serialized_value)
            .with_string_variants()
            .with_struct_map(),
    )?;
    db.prepare_cached(
        // language=sql
        r#"
            -- noinspection SqlResolve @ any/"excluded"
            INSERT INTO readings (sensor_fk, timestamp, value)
            VALUES ((
                SELECT pk
                FROM sensors
                WHERE sensor_id = ?1
            ), ?2, ?3)
            ON CONFLICT (sensor_fk, timestamp) DO UPDATE SET value = excluded.value
        "#,
    )?
    .execute(params![sensor.sensor_id, timestamp, serialized_value])?;

    Ok(())
}

pub fn select_actuals(db: &Connection) -> Result<Vec<Actual>> {
    db.prepare_cached(
        // language=sql
        r#"
            SELECT sensors.sensor_id, sensors.title, sensors.timestamp, sensors.room_title, value
            FROM sensors
            INNER JOIN readings
                ON readings.timestamp = sensors.timestamp AND readings.sensor_fk = sensors.pk
            ORDER BY sensors.room_title, sensors.sensor_id
        "#,
    )?
    .query_map(params![], |row| {
        Ok(Actual {
            sensor: Sensor {
                sensor_id: row.get("sensor_id")?,
                title: row.get("title")?,
                room_title: row.get("room_title")?,
            },
            reading: reading_from_row(row)?,
        })
    })?
    .map(|r| r.map_err(Error::from))
    .collect()
}

/// Select database size in bytes.
pub fn select_size(db: &Connection) -> Result<u64> {
    Ok(db
        // language=sql
        .prepare_cached(
            r#"
            -- noinspection SqlResolve
            SELECT page_count * page_size as size FROM pragma_page_count(), pragma_page_size()
            "#,
        )?
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
        .query_row(params![sensor_id], reading_from_row)
        .optional()?)
}

/// Select the latest sensor readings within the given time interval.
#[allow(dead_code)]
pub fn select_readings(db: &Connection, sensor_id: &str, since: &DateTime<Local>) -> Result<Vec<Reading>> {
    db
        // language=sql
        .prepare_cached(
            r#"
            SELECT readings.timestamp, value
            FROM readings
            INNER JOIN sensors ON sensors.pk = readings.sensor_fk
            WHERE sensors.sensor_id = ?1 AND readings.timestamp >= ?2
            ORDER BY readings.timestamp
            "#,
        )?
        .query_map(params![sensor_id, since.timestamp_millis()], reading_from_row)?
        .map(|r| r.map_err(Error::from))
        .collect()
}

fn reading_from_row(row: &Row) -> rusqlite::Result<Reading> {
    Ok(Reading {
        timestamp: Local.timestamp_millis(row.get("timestamp")?),
        value: rmp_serde::from_read_ref(&row.get::<_, Vec<u8>>("value")?).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Blob, Box::new(error))
        })?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    type Result = crate::Result<()>;

    #[test]
    fn double_upsert_sensor_keeps_one_record() -> Result {
        let sensor = Sensor {
            sensor_id: "test".into(),
            title: None,
            room_title: None,
        };

        let db = connect(":memory:")?;
        upsert_sensor(&db, &sensor, 0)?;
        let sensor_pk = get_sensor_pk(&db, "test")?.unwrap();
        upsert_sensor(&db, &sensor, 0)?;
        assert_eq!(get_sensor_pk(&db, "test")?.unwrap(), sensor_pk);

        Ok(())
    }

    #[test]
    fn double_upsert_reading_keeps_one_record() -> Result {
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
        assert_eq!(
            select_actuals(&db)?,
            vec![Actual {
                sensor: message.sensor,
                reading: message.reading
            }]
        );
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

    #[test]
    fn migrates_to_the_latest_version() -> Result {
        let db = connect(":memory:")?;
        let version = get_version(&db)? as usize;
        assert_eq!(version, migrations::MIGRATIONS.len() - 1);
        Ok(())
    }

    fn get_sensor_pk(db: &Connection, sensor_id: &str) -> crate::Result<Option<i64>> {
        Ok(db
            // language=sql
            .prepare_cached("SELECT pk FROM sensors WHERE sensor_id = ?1")?
            .query_row(params![sensor_id], |row| row.get::<_, i64>(0))
            .optional()?)
    }
}
