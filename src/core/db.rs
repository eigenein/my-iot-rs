//! Database interface.

use crate::prelude::*;
use chrono::prelude::*;
use rusqlite::types::FromSql;
use rusqlite::{params, Row};
use rusqlite::{OptionalExtension, NO_PARAMS};
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

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
        connection
            .connection()?
            .execute("PRAGMA foreign_keys = ON", NO_PARAMS)?;
        connection.migrate()?;
        Ok(connection)
    }

    fn migrate(&self) -> Result {
        let user_version = self.get_user_version()?;
        let mut connection = self.connection()?;
        for (i, migration) in MIGRATIONS.iter().enumerate() {
            if user_version < i + 1 {
                info!("Applying migration #{}…", i + 1);
                let tx = connection.transaction()?;
                tx.execute_batch(migration)?;
                tx.commit()?;
                info!("Vacuuming…");
                connection.execute("VACUUM", NO_PARAMS)?;
            }
        }
        Ok(())
    }

    /// Acquires lock and returns the underlying `rusqlite::Connection`.
    pub fn connection(&self) -> Result<MutexGuard<'_, rusqlite::Connection>> {
        Ok(self.connection.lock().expect("Failed to acquire the database lock"))
    }

    pub fn get_user_version(&self) -> Result<usize> {
        Ok(self
            .connection()?
            .pragma_query_value(None, "user_version", get_single::<i64, usize>)?)
    }

    /// Selects the latest readings for all sensors.
    pub fn select_actuals(&self) -> Result<Vec<(Sensor, Reading)>> {
        self.connection()?
            .prepare_cached(
                // language=sql
                r"SELECT * FROM sensors ORDER BY location, sensor_id",
            )?
            .query_map(NO_PARAMS, get_sensor_reading)?
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
            .query_row(NO_PARAMS, get_single::<i64, u64>)?)
    }

    /// Selects the specified sensor.
    pub fn select_sensor(&self, sensor_id: &str) -> Result<Option<(Sensor, Reading)>> {
        Ok(self
            .connection()?
            // language=sql
            .prepare_cached(r"SELECT * FROM sensors WHERE sensor_id = ?1")?
            .query_row(params![sensor_id], get_sensor_reading)
            .optional()?)
    }

    pub fn delete_sensor(&self, sensor_id: &str) -> Result {
        self.connection()?
            // language=sql
            .prepare_cached(r"DELETE FROM sensors WHERE sensor_id = ?1")?
            .execute(params![sensor_id])?;
        Ok(())
    }

    /// Selects the specified sensor readings within the specified period.
    pub fn select_readings(&self, sensor_id: &str, since: &DateTime<Local>) -> Result<Vec<Reading>> {
        self.connection()?
            // language=sql
            .prepare_cached(
                r#"
                SELECT timestamp, value
                FROM readings
                WHERE sensor_fk = ?1 AND timestamp >= ?2
                ORDER BY timestamp
                "#,
            )?
            .query_map(
                params![hash_sensor_id(sensor_id), since.timestamp_millis()],
                get_reading,
            )?
            .map(|r| r.map_err(Into::into))
            .collect()
    }

    pub fn select_last_n_readings(&self, sensor_id: &str, limit: usize) -> Result<Vec<Reading>> {
        self.connection()?
            // language=sql
            .prepare_cached("SELECT timestamp, value FROM readings WHERE sensor_fk = ?1 ORDER BY timestamp LIMIT ?2")?
            .query_map(params![hash_sensor_id(sensor_id), limit as i64], get_reading)?
            .map(|r| r.map_err(Into::into))
            .collect()
    }

    pub fn select_sensor_count(&self) -> Result<usize> {
        Ok(self
            .connection()?
            // language=sql
            .prepare_cached("SELECT COUNT(*) FROM sensors")?
            .query_row(NO_PARAMS, get_single::<i64, usize>)?)
    }

    pub fn select_reading_count(&self) -> Result<u64> {
        Ok(self
            .connection()?
            // language=sql
            .prepare_cached("SELECT COUNT(*) FROM readings")?
            .query_row(NO_PARAMS, get_single::<i64, u64>)?)
    }

    pub fn select_sensor_reading_count(&self, sensor_id: &str) -> Result<u64> {
        Ok(self
            .connection()?
            // language=sql
            .prepare_cached("SELECT COUNT(*) FROM readings WHERE sensor_fk = ?1")?
            .query_row(params![hash_sensor_id(sensor_id)], get_single::<i64, u64>)?)
    }

    pub fn set_user_data<V: Serialize>(&self, key: &str, value: V, expires_at: Option<DateTime<Local>>) -> Result {
        self.connection()?
            // language=sql
            .prepare_cached(
                r#"
                -- noinspection SqlResolve @ any/"excluded"
                INSERT INTO user_data (pk, value, expires_at)
                VALUES (?1, ?2, ?3)
                ON CONFLICT (pk) DO UPDATE SET value = excluded.value, expires_at = excluded.expires_at
            "#,
            )?
            .execute(params![
                key,
                bincode::serialize(&value)?,
                expires_at.as_ref().map(DateTime::<Local>::timestamp_millis),
            ])?;
        Ok(())
    }

    pub fn get_user_data<V: DeserializeOwned>(&self, key: &str) -> Result<Option<V>> {
        Ok(self
            .connection()?
            // language=sql
            .prepare_cached(
                r#"
                -- Having fun with strings getting auto-converted to integers.
                SELECT value FROM user_data
                WHERE pk = ?1 AND (expires_at IS NULL OR expires_at >= ?2)
                "#,
            )?
            .query_row(params![key, Local::now().timestamp_millis()], |row| {
                Ok(bincode::deserialize(&row.get::<_, Vec<u8>>(0)?).unwrap())
            })
            .optional()?)
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
        location: row.get("location")?,
        is_writable: row.get("is_writable")?,
    })
}

/// Builds a `Reading` instance based on the database row.
fn get_reading(row: &Row) -> rusqlite::Result<Reading> {
    Ok(Reading {
        timestamp: Local.timestamp_millis(row.get("timestamp")?),
        value: bincode::deserialize(&row.get::<_, Vec<u8>>("value")?).unwrap_or(Value::Other),
    })
}

fn get_sensor_reading(row: &Row) -> rusqlite::Result<(Sensor, Reading)> {
    Ok((get_sensor(row)?, get_reading(row)?))
}

/// Gets a single value from the row.
///
/// # Type Arguments
///
/// - `T`: type that is passed to the database driver.
/// - `R`: desired return type.
#[inline(always)]
fn get_single<T, R>(row: &Row) -> rusqlite::Result<R>
where
    T: FromSql,
    R: TryFrom<T>,
    R::Error: Send + Sync + Error + 'static,
{
    TryInto::<R>::try_into(row.get::<_, T>(0)?)
        .map_err(Box::new)
        .map_err(|error| rusqlite::Error::FromSqlConversionFailure(0, row.get_raw(0).data_type(), error))
}

impl Message {
    /// Upsert the message into the database.
    pub fn upsert_into(&self, connection: &rusqlite::Connection) -> Result {
        let sensor_pk = hash_sensor_id(&self.sensor.id);
        let timestamp = self.reading.timestamp.timestamp_millis();
        let value = bincode::serialize(&self.reading.value)?;

        connection
            .prepare_cached(
                // language=sql
                r#"
                    -- noinspection SqlResolve @ any/"excluded"
                    INSERT INTO sensors (pk, sensor_id, title, timestamp, location, value, is_writable)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                    ON CONFLICT (pk) DO UPDATE SET
                        timestamp = excluded.timestamp,
                        title = excluded.title,
                        location = excluded.location,
                        value = excluded.value,
                        is_writable = excluded.is_writable
                "#,
            )?
            .execute(params![
                sensor_pk,
                self.sensor.id,
                self.sensor.title,
                timestamp,
                self.sensor.location,
                value,
                self.sensor.is_writable,
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

// TODO: move to a separate file.
const MIGRATIONS: &[&str] = &[
    // language=sql
    r#"
    CREATE TABLE IF NOT EXISTS sensors (
        pk INTEGER NOT NULL PRIMARY KEY, -- `sensor_id` SeaHash
        sensor_id TEXT NOT NULL UNIQUE,
        timestamp INTEGER NOT NULL, -- unix time, milliseconds
        title TEXT DEFAULT NULL,
        room_title TEXT DEFAULT NULL, -- renamed to `location`
        value JSON NOT NULL,
        expires_at INTEGER NOT NULL -- unused
    );

    CREATE TABLE IF NOT EXISTS readings (
        sensor_fk INTEGER NOT NULL REFERENCES sensors ON UPDATE CASCADE ON DELETE CASCADE,
        timestamp INTEGER NOT NULL, -- unix time, milliseconds
        value JSON NOT NULL -- serialized `Value`
    );

    CREATE UNIQUE INDEX IF NOT EXISTS readings_sensor_fk_timestamp
        ON readings (sensor_fk ASC, timestamp DESC);

    CREATE TABLE IF NOT EXISTS user_data (
        pk TEXT NOT NULL PRIMARY KEY,
        value JSON NOT NULL,
        expires_at INTEGER NULL -- unix time, milliseconds
    );

    PRAGMA user_version = 1;
    "#,
    //
    // language=sql
    r#"
    ALTER TABLE sensors ADD COLUMN is_writable INTEGER NOT NULL DEFAULT 0;
    PRAGMA user_version = 2;
    "#,
    //
    // language=sql
    r#"
    DROP TABLE readings;
    DROP TABLE sensors;
    DROP TABLE user_data;

    CREATE TABLE IF NOT EXISTS sensors (
        pk INTEGER NOT NULL PRIMARY KEY, -- `sensor_id` SeaHash
        sensor_id TEXT NOT NULL UNIQUE,
        timestamp INTEGER NOT NULL, -- unix time, milliseconds
        title TEXT DEFAULT NULL,
        location TEXT DEFAULT NULL,
        value BLOB NOT NULL,
        is_writable INTEGER NOT NULL
    );

    CREATE TABLE IF NOT EXISTS readings (
        sensor_fk INTEGER NOT NULL REFERENCES sensors ON UPDATE CASCADE ON DELETE CASCADE,
        timestamp INTEGER NOT NULL, -- unix time, milliseconds
        value JSON NOT NULL -- serialized `Value`
    );

    CREATE UNIQUE INDEX IF NOT EXISTS readings_sensor_fk_timestamp
        ON readings (sensor_fk ASC, timestamp DESC);

    CREATE TABLE IF NOT EXISTS user_data (
        pk TEXT NOT NULL PRIMARY KEY,
        value BLOB NOT NULL,
        expires_at INTEGER NULL -- unix time, milliseconds
    );

    PRAGMA user_version = 3;
    "#,
];

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

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
    fn select_readings_ok() -> Result {
        let db = Connection::open_and_initialize(":memory:")?;
        let message = Message::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000));
        message.upsert_into(&*db.connection()?)?;
        let readings = db.select_readings("test", &Local.timestamp_millis(0))?;
        assert_eq!(readings.get(0).unwrap(), &message.reading);
        Ok(())
    }

    #[test]
    fn get_set_user_data_ok() -> Result {
        let db = Connection::open_and_initialize(":memory:")?;
        db.set_user_data("hello::world", 42_i32, Some(Local::now() + Duration::minutes(1)))?;
        assert_eq!(db.get_user_data("hello::world")?, Some(42_i32));
        Ok(())
    }

    #[test]
    fn get_set_user_data_overwrite_ok() -> Result {
        let db = Connection::open_and_initialize(":memory:")?;
        db.set_user_data("hello::world", 43_i32, None)?;
        db.set_user_data("hello::world", 42_i32, None)?;
        assert_eq!(db.get_user_data("hello::world")?, Some(42_i32));
        Ok(())
    }

    #[test]
    fn get_expired_user_data_ok() -> Result {
        let db = Connection::open_and_initialize(":memory:")?;
        db.set_user_data("hello::world", 43_i32, Some(Local::now() - Duration::minutes(1)))?;
        assert_eq!(db.get_user_data::<i32>("hello::world")?, None);
        Ok(())
    }

    #[test]
    fn missing_user_data_returns_none() -> Result {
        let db = Connection::open_and_initialize(":memory:")?;
        assert_eq!(db.get_user_data::<String>("hello::world")?, None);
        Ok(())
    }
}
