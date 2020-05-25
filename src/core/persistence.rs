//! Database interface.

use crate::prelude::*;
use chrono::prelude::*;
use rusqlite::{params, Row};
use rusqlite::{OptionalExtension, NO_PARAMS};
use std::path::Path;

mod migrations;
mod primitives;
pub mod reading;
pub mod sensor;
pub mod thread;

impl From<Message> for (Sensor, Reading) {
    fn from(message: Message) -> Self {
        (message.sensor, message.reading)
    }
}

pub trait ConnectionExtensions {
    fn open_and_initialize<P: AsRef<Path>>(path: P) -> Result<Connection>;

    /// Get the database `user_version`.
    fn get_version(&self) -> Result<i32>;

    fn select_actuals(&self, max_sensor_age_ms: i64) -> Result<Vec<(Sensor, Reading)>>;

    /// Select database size in bytes.
    fn select_size(&self) -> Result<u64>;

    /// Select the very last sensor reading.
    fn select_last_reading(&self, sensor_id: &str) -> Result<Option<(Sensor, Reading)>>;

    /// Select the latest sensor readings within the given time interval.
    fn select_readings(&self, sensor_id: &str, since: &DateTime<Local>) -> Result<Vec<Reading>>;

    fn select_sensor_count(&self) -> Result<u64>;

    fn select_reading_count(&self) -> Result<u64>;
}

impl ConnectionExtensions for Connection {
    fn open_and_initialize<P: AsRef<Path>>(path: P) -> Result<Connection> {
        let mut db = Connection::open(path)?;
        // language=sql
        db.execute_batch("PRAGMA foreign_keys = ON;")?;
        migrate(&mut db)?;
        Ok(db)
    }

    fn get_version(&self) -> Result<i32> {
        let version: i32 = self.pragma_query_value(None, "user_version", |row| row.get(0))?;
        Ok(version)
    }

    fn select_actuals(&self, max_sensor_age_ms: i64) -> Result<Vec<(Sensor, Reading)>> {
        self.prepare_cached(
            // language=sql
            r#"
            SELECT sensor_id, title, timestamp, room_title, value
            FROM sensors
            WHERE timestamp > ?1
            ORDER BY sensors.room_title, sensors.sensor_id
            "#,
        )?
        .query_map(
            params![Local::now().timestamp_millis() - max_sensor_age_ms as i64],
            get_actual,
        )?
        .map(|r| r.map_err(Into::into))
        .collect()
    }

    fn select_size(&self) -> Result<u64> {
        Ok(self
            // language=sql
            .prepare_cached(
                r#"
                -- noinspection SqlResolve
                SELECT page_count * page_size as size FROM pragma_page_count(), pragma_page_size()
                "#,
            )?
            .query_row(NO_PARAMS, get_i64)
            .map(|v| v as u64)?)
    }

    fn select_last_reading(&self, sensor_id: &str) -> Result<Option<(Sensor, Reading)>> {
        Ok(self
            // language=sql
            .prepare_cached(r#"SELECT * FROM sensors WHERE sensors.sensor_id = ?1"#)?
            .query_row(params![sensor_id], get_actual)
            .optional()?)
    }

    #[allow(dead_code)]
    fn select_readings(&self, sensor_id: &str, since: &DateTime<Local>) -> Result<Vec<Reading>> {
        self
            // language=sql
            .prepare_cached(
                r#"
                SELECT readings.timestamp, readings.value
                FROM readings
                INNER JOIN sensors ON sensors.pk = readings.sensor_fk
                WHERE sensors.sensor_id = ?1 AND readings.timestamp >= ?2
                ORDER BY readings.timestamp
                "#,
            )?
            .query_map(params![sensor_id, since.timestamp_millis()], get_reading)?
            .map(|r| r.map_err(Into::into))
            .collect()
    }

    fn select_sensor_count(&self) -> Result<u64> {
        Ok(self
            // language=sql
            .prepare("SELECT COUNT(*) FROM sensors")?
            .query_row(NO_PARAMS, get_i64)
            .map(|v| v as u64)?)
    }

    fn select_reading_count(&self) -> Result<u64> {
        Ok(self
            // language=sql
            .prepare("SELECT COUNT(*) FROM readings")?
            .query_row(NO_PARAMS, get_i64)
            .map(|v| v as u64)?)
    }
}

fn migrate(db: &mut Connection) -> Result<()> {
    let version = db.get_version()? as usize;
    for (i, migrate) in migrations::MIGRATIONS.iter().enumerate() {
        if version < i {
            info!("Migrating to version {}…", i);
            let tx = db.transaction()?;
            migrate(&tx)?;
            tx.pragma_update(None, "user_version", &(i as i32))?;
            tx.commit()?;
        }
    }
    Ok(())
}

fn get_sensor(row: &Row) -> rusqlite::Result<Sensor> {
    Ok(Sensor {
        id: row.get("sensor_id")?,
        title: row.get("title")?,
        room_title: row.get("room_title")?,
    })
}

fn get_reading(row: &Row) -> rusqlite::Result<Reading> {
    Ok(Reading {
        timestamp: Local.timestamp_millis(row.get("timestamp")?),
        value: rmp_serde::from_read_ref(&row.get::<_, Vec<u8>>("value")?).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Blob, Box::new(error))
        })?,
    })
}

fn get_actual(row: &Row) -> rusqlite::Result<(Sensor, Reading)> {
    Ok((get_sensor(row)?, get_reading(row)?))
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
            self.sensor.id,
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
        .execute(params![self.sensor.id, timestamp, value])?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Result = crate::Result<()>;

    #[test]
    fn double_upsert_keeps_one_reading() -> Result {
        let message = Message::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000));

        let db = Connection::open_and_initialize(":memory:")?;
        message.upsert_into(&db)?;
        message.upsert_into(&db)?;

        assert_eq!(db.select_reading_count()?, 1);

        Ok(())
    }

    #[test]
    fn select_last_reading_returns_none_on_empty_database() -> Result {
        let db = Connection::open_and_initialize(":memory:")?;
        assert_eq!(db.select_last_reading("test")?, None);
        Ok(())
    }

    #[test]
    fn select_last_reading_ok() -> Result {
        let message = Message::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000));
        let db = Connection::open_and_initialize(":memory:")?;
        message.upsert_into(&db)?;
        assert_eq!(db.select_last_reading("test")?, Some(message.into()));
        Ok(())
    }

    #[test]
    fn select_last_reading_returns_newer_reading() -> Result {
        let db = Connection::open_and_initialize(":memory:")?;
        let old = Message::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_127_000));
        old.upsert_into(&db)?;
        let new = Message::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000));
        new.upsert_into(&db)?;
        assert_eq!(db.select_last_reading("test")?, Some(new.into()));
        Ok(())
    }

    #[test]
    fn select_actuals_ok() -> Result {
        let message = Message::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000));
        let db = Connection::open_and_initialize(":memory:")?;
        message.upsert_into(&db)?;
        assert_eq!(
            db.select_actuals(i64::max_value())?,
            vec![(message.sensor, message.reading)]
        );
        Ok(())
    }

    #[test]
    fn existing_sensor_is_reused() -> Result {
        let db = Connection::open_and_initialize(":memory:")?;
        let old = Message::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000));
        old.upsert_into(&db)?;
        let new = Message::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_129_000));
        new.upsert_into(&db)?;

        assert_eq!(db.select_sensor_count()?, 1);

        Ok(())
    }

    #[test]
    fn migrates_to_the_latest_version() -> Result {
        let db = Connection::open_and_initialize(":memory:")?;
        let version = db.get_version()? as usize;
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
