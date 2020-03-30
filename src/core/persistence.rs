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

pub fn select_actuals(db: &Connection) -> Result<Vec<Actual>> {
    db.prepare_cached(
        // language=sql
        r#"
            SELECT sensor_id, title, timestamp, room_title, value
            FROM sensors
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
            reading: get_reading(row)?,
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
        .query_row(params![], get_i64)
        .map(|v| v as u64)?)
}

/// Select the very last sensor reading.
pub fn select_last_reading(db: &Connection, sensor_id: &str) -> Result<Option<Reading>> {
    Ok(db
        // language=sql
        .prepare_cached(
            r#"
            SELECT timestamp, value
            FROM sensors
            WHERE sensors.sensor_id = ?1
            "#,
        )?
        .query_row(params![sensor_id], get_reading)
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
        .query_map(params![sensor_id, since.timestamp_millis()], get_reading)?
        .map(|r| r.map_err(Error::from))
        .collect()
}

fn get_reading(row: &Row) -> rusqlite::Result<Reading> {
    Ok(Reading {
        timestamp: Local.timestamp_millis(row.get("timestamp")?),
        value: rmp_serde::from_read_ref(&row.get::<_, Vec<u8>>("value")?).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Blob, Box::new(error))
        })?,
    })
}

fn get_i64(row: &Row) -> rusqlite::Result<i64> {
    row.get::<_, i64>(0)
}

impl Value {
    fn serialize_to_vec(&self) -> Result<Vec<u8>> {
        let mut serialized_value = Vec::new();
        self.serialize(
            &mut rmp_serde::Serializer::new(&mut serialized_value)
                .with_string_variants()
                .with_struct_map(),
        )?;
        Ok(serialized_value)
    }
}

impl Message {
    pub fn upsert_into(&self, db: &Connection) -> Result<()> {
        let timestamp = self.reading.timestamp.timestamp_millis();
        let value = self.reading.value.serialize_to_vec()?;

        db.prepare_cached(
            // language=sql
            r#"
            -- noinspection SqlResolve @ any/"excluded"
            -- noinspection SqlInsertValues
            INSERT INTO sensors (sensor_id, title, timestamp, room_title, value) VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT (sensor_id) DO UPDATE SET
                timestamp = excluded.timestamp,
                title = excluded.title,
                room_title = excluded.room_title,
                value = excluded.value
            "#,
        )?
        .execute(params![
            self.sensor.sensor_id,
            self.sensor.title,
            timestamp,
            self.sensor.room_title,
            value,
        ])?;

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
        .execute(params![self.sensor.sensor_id, timestamp, value])?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Result = crate::Result<()>;

    #[test]
    fn double_upsert_keeps_one_reading() -> Result {
        let message = Composer::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000))
            .message;

        let db = connect(":memory:")?;
        message.upsert_into(&db)?;
        message.upsert_into(&db)?;

        let reading_count: i64 = db
            // language=sql
            .prepare("SELECT COUNT(*) FROM readings")?
            .query_row(params![], get_i64)?;
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
        message.upsert_into(&db)?;
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
        old.upsert_into(&db)?;
        let new = Composer::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000))
            .message;
        new.upsert_into(&db)?;
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
        message.upsert_into(&db)?;
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
        old.upsert_into(&db)?;
        let new = Composer::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_129_000))
            .message;
        new.upsert_into(&db)?;

        let reading_count = db
            // language=sql
            .prepare_cached("SELECT COUNT(*) FROM sensors")?
            .query_row(params![], get_i64)?;
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

    #[allow(dead_code)]
    fn get_sensor_pk(db: &Connection, sensor_id: &str) -> crate::Result<Option<i64>> {
        Ok(db
            // language=sql
            .prepare_cached("SELECT pk FROM sensors WHERE sensor_id = ?1")?
            .query_row(params![sensor_id], get_i64)
            .optional()?)
    }
}
