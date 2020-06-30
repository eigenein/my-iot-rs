//! Database interface.

use crate::prelude::*;
use chrono::prelude::*;
use rusqlite::types::FromSql;
use rusqlite::{params, Row};
use rusqlite::{OptionalExtension, NO_PARAMS};
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

mod migrations;
pub mod reading;
pub mod sensor;
pub mod thread;

/// Wraps `rusqlite::Connection` and provides the high-level database methods.
#[derive(Clone)]
pub struct Connection {
    connection: Arc<Mutex<rusqlite::Connection>>,
}

impl Connection {
    pub fn open_and_initialize<P: AsRef<Path>>(path: P) -> Result<Self> {
        let connection = Self {
            connection: Arc::new(Mutex::new(rusqlite::Connection::open(path)?)),
        };
        // language=sql
        connection.connection()?.execute_batch("PRAGMA foreign_keys = ON;")?;
        connection.migrate()?;
        Ok(connection)
    }

    /// Acquires lock and returns the underlying `rusqlite::Connection`.
    pub fn connection(&self) -> Result<MutexGuard<'_, rusqlite::Connection>> {
        Ok(self.connection.lock().expect("Failed to acquire the database lock"))
    }

    /// Selects the latest readings for all sensors.
    pub fn select_actuals(&self) -> Result<Vec<(Sensor, Reading)>> {
        self.connection()?
            .prepare_cached(
                // language=sql
                r"SELECT * FROM sensors ORDER BY room_title, sensor_id",
            )?
            .query_map(NO_PARAMS, get_actual)?
            .map(|r| r.map_err(Into::into))
            .collect()
    }

    /// Selects the database size.
    pub fn select_size(&self) -> Result<u64> {
        Ok(self
            .connection()?
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

    /// Selects the specified sensor.
    pub fn select_sensor(&self, sensor_id: &str) -> Result<Option<(Sensor, Reading)>> {
        Ok(self
            .connection()?
            // language=sql
            .prepare_cached(r"SELECT * FROM sensors WHERE sensor_id = ?1")?
            .query_row(params![sensor_id], get_actual)
            .optional()?)
    }

    /// Selects the specified sensor readings within the specified period.
    pub fn select_values<T: FromSql>(
        &self,
        sensor_id: &str,
        since: &DateTime<Local>,
    ) -> Result<Vec<(DateTime<Local>, T)>> {
        self.connection()?
            // language=sql
            .prepare_cached(
                r#"
                -- noinspection SqlResolve @ routine/"json_extract"
                SELECT timestamp, json_extract(value, '$.value') as value
                FROM readings
                WHERE sensor_fk = ?1 AND timestamp >= ?2
                ORDER BY timestamp
                "#,
            )?
            .query_map(
                params![hash_sensor_id(sensor_id), since.timestamp_millis()],
                |row| -> rusqlite::Result<(DateTime<Local>, T)> {
                    Ok((Local.timestamp_millis(row.get("timestamp")?), row.get::<_, T>("value")?))
                },
            )?
            .map(|r| r.map_err(Into::into))
            .collect()
    }

    pub fn select_sensor_count(&self) -> Result<u64> {
        Ok(self
            .connection()?
            // language=sql
            .prepare("SELECT COUNT(*) FROM sensors")?
            .query_row(NO_PARAMS, get_i64)
            .map(|v| v as u64)?)
    }

    pub fn select_reading_count(&self) -> Result<u64> {
        Ok(self
            .connection()?
            // language=sql
            .prepare("SELECT COUNT(*) FROM readings")?
            .query_row(NO_PARAMS, get_i64)
            .map(|v| v as u64)?)
    }

    pub fn select_sensor_reading_count(&self, sensor_id: &str) -> Result<u64> {
        Ok(self
            .connection()?
            // language=sql
            .prepare("SELECT COUNT(*) FROM readings WHERE sensor_fk = ?1")?
            .query_row(params![hash_sensor_id(sensor_id)], get_i64)
            .map(|v| v as u64)?)
    }

    /// Select the database version.
    pub fn select_version(&self) -> Result<i32> {
        let version: i32 = self
            .connection()?
            .pragma_query_value(None, "user_version", |row| row.get(0))?;
        Ok(version)
    }

    fn migrate(&self) -> Result<()> {
        let version = self.select_version()? as usize;
        let mut connection = self.connection()?;
        migrations::MIGRATIONS
            .iter()
            .enumerate()
            .filter(|(i, _)| version < *i)
            .map(|(i, migrate)| -> Result<()> {
                info!("Migrating to version {}…", i);
                let tx = connection.transaction()?;
                migrate(&tx)?;
                tx.pragma_update(None, "user_version", &(i as i32))?;
                tx.commit()?;
                connection.execute("VACUUM", NO_PARAMS)?;
                Ok(())
            })
            .collect()
    }
}

/// Hashes the sensor ID, hash is then used for a sensor primary key.
pub fn hash_sensor_id(sensor_id: &str) -> i64 {
    signed_seahash(sensor_id.as_bytes())
}

/// Returns SeaHash of the buffer as a signed integer, because SQLite wants signed integers.
fn signed_seahash(buffer: &[u8]) -> i64 {
    seahash::hash(buffer) as i64
}

/// Builds a `Sensor` instance based on the database row.
fn get_sensor(row: &Row) -> rusqlite::Result<Sensor> {
    Ok(Sensor {
        id: row.get("sensor_id")?,
        title: row.get("title")?,
        room_title: row.get("room_title")?,
    })
}

/// Builds a `Reading` instance based on the database row.
fn get_reading(row: &Row) -> rusqlite::Result<Reading> {
    Ok(Reading {
        timestamp: Local.timestamp_millis(row.get("timestamp")?),
        value: serde_json::from_str(&row.get::<_, String>("value")?).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Blob, Box::new(error))
        })?,
    })
}

fn get_actual(row: &Row) -> rusqlite::Result<(Sensor, Reading)> {
    Ok((get_sensor(row)?, get_reading(row)?))
}

/// Selects a single `i64` value, used with single-integer `SELECT`s.
#[inline(always)]
fn get_i64(row: &Row) -> rusqlite::Result<i64> {
    row.get::<_, i64>(0)
}

impl Message {
    /// Upsert the message into the database.
    pub fn upsert_into(&self, connection: &rusqlite::Connection) -> Result<()> {
        let sensor_pk = hash_sensor_id(&self.sensor.id);
        let timestamp = self.reading.timestamp.timestamp_millis();
        let value = serde_json::to_string(&self.reading.value)?;

        connection
            .prepare_cached(
                // language=sql
                r#"
            -- noinspection SqlResolve @ any/"excluded"
            -- noinspection SqlInsertValues
            INSERT INTO sensors (pk, sensor_id, title, timestamp, room_title, value, expires_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0)
            ON CONFLICT (pk) DO UPDATE SET
                timestamp = excluded.timestamp,
                title = excluded.title,
                room_title = excluded.room_title,
                value = excluded.value
            "#,
            )?
            .execute(params![
                sensor_pk,
                self.sensor.id,
                self.sensor.title,
                timestamp,
                self.sensor.room_title,
                value,
            ])?;

        connection
            .prepare_cached(
                // language=sql
                r#"
            -- noinspection SqlResolve @ any/"excluded"
            INSERT INTO readings (sensor_fk, timestamp, value)
            VALUES (?1, ?2, ?3)
            ON CONFLICT (sensor_fk, timestamp) DO UPDATE SET value = excluded.value
            "#,
            )?
            .execute(params![sensor_pk, timestamp, value])?;

        Ok(())
    }
}

impl From<Message> for (Sensor, Reading) {
    fn from(message: Message) -> Self {
        (message.sensor, message.reading)
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
        {
            // It acquires a lock on the database.
            let connection = db.connection()?;
            message.upsert_into(&connection)?;
            message.upsert_into(&connection)?;
        }

        assert_eq!(db.select_reading_count()?, 1);

        Ok(())
    }

    #[test]
    fn select_last_reading_returns_none_on_empty_database() -> Result {
        let db = Connection::open_and_initialize(":memory:")?;
        assert_eq!(db.select_sensor("test")?, None);
        Ok(())
    }

    #[test]
    fn select_last_reading_ok() -> Result {
        let message = Message::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000));
        let db = Connection::open_and_initialize(":memory:")?;
        message.upsert_into(&*db.connection()?)?;
        assert_eq!(db.select_sensor("test")?, Some(message.into()));
        Ok(())
    }

    #[test]
    fn select_last_reading_returns_newer_reading() -> Result {
        let db = Connection::open_and_initialize(":memory:")?;
        let mut message = Message::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_127_000));
        message.upsert_into(&*db.connection()?)?;
        message = message.timestamp(Local.timestamp_millis(1_566_424_128_000));
        message.upsert_into(&*db.connection()?)?;
        assert_eq!(db.select_sensor("test")?, Some(message.into()));
        Ok(())
    }

    #[test]
    fn select_actuals_ok() -> Result {
        let message = Message::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000));
        let db = Connection::open_and_initialize(":memory:")?;
        message.upsert_into(&*db.connection()?)?;
        assert_eq!(db.select_actuals()?, vec![(message.sensor, message.reading)]);
        Ok(())
    }

    #[test]
    fn existing_sensor_is_reused() -> Result {
        let db = Connection::open_and_initialize(":memory:")?;
        let old = Message::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000));
        old.upsert_into(&*db.connection()?)?;
        let new = Message::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_129_000));
        new.upsert_into(&*db.connection()?)?;

        assert_eq!(db.select_sensor_count()?, 1);

        Ok(())
    }

    #[test]
    fn migrates_to_the_latest_version() -> Result {
        let db = Connection::open_and_initialize(":memory:")?;
        let version = db.select_version()? as usize;
        assert_eq!(version, migrations::MIGRATIONS.len() - 1);
        Ok(())
    }

    #[test]
    fn select_readings_ok() -> Result {
        let db = Connection::open_and_initialize(":memory:")?;
        let message = Message::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000));
        message.upsert_into(&*db.connection()?)?;
        let readings: Vec<(_, i64)> = db.select_values("test", &Local.timestamp_millis(0))?;
        assert_eq!(readings.get(0).unwrap(), &(message.reading.timestamp, 42));
        Ok(())
    }
}
