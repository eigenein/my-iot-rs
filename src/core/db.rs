//! Database interface.

use crate::prelude::*;
use chrono::prelude::*;
use rusqlite::types::FromSql;
use rusqlite::{params, Row};
use rusqlite::{OptionalExtension, NO_PARAMS};
use std::path::Path;

pub mod migrations;
pub mod reading;
pub mod sensor;
pub mod tasks;

/// Wraps `rusqlite::Connection` and provides the high-level database methods.
#[derive(Clone)]
pub struct Connection {
    connection: Arc<Mutex<rusqlite::Connection>>,
}

impl Connection {
    pub async fn open_and_initialize<P: AsRef<Path>>(path: P) -> Result<Self> {
        let connection = Self {
            connection: Arc::new(Mutex::new(rusqlite::Connection::open(path)?)),
        };
        connection
            .connection()
            .await
            .execute("PRAGMA foreign_keys = ON", NO_PARAMS)?;
        connection.migrate().await?;
        Ok(connection)
    }

    async fn migrate(&self) -> Result {
        let user_version = self.get_user_version().await?;
        let mut connection = self.connection().await;
        for (i, migration) in migrations::MIGRATIONS.iter().enumerate() {
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

    pub async fn connection(&self) -> MutexGuard<'_, rusqlite::Connection> {
        self.connection.lock().await
    }

    pub async fn get_user_version(&self) -> Result<usize> {
        Ok(self
            .connection()
            .await
            .pragma_query_value(None, "user_version", get_single::<i64, usize>)?)
    }

    /// Selects the latest readings for all sensors.
    pub async fn select_actuals(&self) -> Result<Vec<(Sensor, Reading)>> {
        self.connection()
            .await
            .prepare_cached(
                // language=sql
                r"SELECT * FROM sensors ORDER BY location, sensor_id",
            )?
            .query_map(NO_PARAMS, get_sensor_reading)?
            .map(|r| r.map_err(Into::into))
            .collect()
    }

    /// Selects the database size.
    pub async fn select_size(&self) -> Result<u64> {
        Ok(self
            .connection()
            .await
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
    pub async fn select_sensor(&self, sensor_id: &str) -> Result<Option<(Sensor, Reading)>> {
        Ok(self
            .connection()
            .await
            // language=sql
            .prepare_cached(r"SELECT * FROM sensors WHERE sensor_id = ?1")?
            .query_row(params![sensor_id], get_sensor_reading)
            .optional()?)
    }

    pub async fn delete_sensor(&self, sensor_id: &str) -> Result {
        self.connection()
            .await
            // language=sql
            .prepare_cached(r"DELETE FROM sensors WHERE sensor_id = ?1")?
            .execute(params![sensor_id])?;
        Ok(())
    }

    /// Selects the specified sensor readings within the specified period.
    pub async fn select_readings(&self, sensor_id: &str, since: &DateTime<Local>) -> Result<Vec<Reading>> {
        self.connection()
            .await
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

    pub async fn select_last_n_readings(&self, sensor_id: &str, limit: usize) -> Result<Vec<Reading>> {
        self.connection()
            .await
            // language=sql
            .prepare_cached("SELECT timestamp, value FROM readings WHERE sensor_fk = ?1 ORDER BY timestamp LIMIT ?2")?
            .query_map(params![hash_sensor_id(sensor_id), limit as i64], get_reading)?
            .map(|r| r.map_err(Into::into))
            .collect()
    }

    pub async fn select_sensor_count(&self) -> Result<usize> {
        Ok(self
            .connection()
            .await
            // language=sql
            .prepare_cached("SELECT COUNT(*) FROM sensors")?
            .query_row(NO_PARAMS, get_single::<i64, usize>)?)
    }

    pub async fn select_reading_count(&self) -> Result<u64> {
        Ok(self
            .connection()
            .await
            // language=sql
            .prepare_cached("SELECT COUNT(*) FROM readings")?
            .query_row(NO_PARAMS, get_single::<i64, u64>)?)
    }

    pub async fn select_sensor_reading_count(&self, sensor_id: &str) -> Result<u64> {
        Ok(self
            .connection()
            .await
            // language=sql
            .prepare_cached("SELECT COUNT(*) FROM readings WHERE sensor_fk = ?1")?
            .query_row(params![hash_sensor_id(sensor_id)], get_single::<i64, u64>)?)
    }

    pub async fn set_user_data<V: Serialize>(
        &self,
        key: &str,
        value: V,
        expires_at: Option<DateTime<Local>>,
    ) -> Result {
        self.connection()
            .await
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

    pub async fn get_user_data<V: DeserializeOwned>(&self, key: &str) -> Result<Option<V>> {
        Ok(self
            .connection()
            .await
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
    R::Error: Send + Sync + std::error::Error + 'static,
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[async_std::test]
    async fn double_upsert_keeps_one_reading() -> Result {
        let message = Message::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000));

        let db = Connection::open_and_initialize(":memory:").await?;
        {
            // It acquires a lock on the database.
            let connection = db.connection.lock().await;
            message.upsert_into(&connection)?;
            message.upsert_into(&connection)?;
        }

        assert_eq!(db.select_reading_count().await?, 1);

        Ok(())
    }

    #[async_std::test]
    async fn select_last_reading_returns_none_on_empty_database() -> Result {
        let db = Connection::open_and_initialize(":memory:").await?;
        assert_eq!(db.select_sensor("test").await?, None);
        Ok(())
    }

    #[async_std::test]
    async fn select_last_reading_ok() -> Result {
        let message = Message::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000));
        let db = Connection::open_and_initialize(":memory:").await?;
        message.upsert_into(&*db.connection.lock().await)?;
        assert_eq!(db.select_sensor("test").await?, Some(message.into()));
        Ok(())
    }

    #[async_std::test]
    async fn select_last_reading_returns_newer_reading() -> Result {
        let db = Connection::open_and_initialize(":memory:").await?;
        let mut message = Message::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_127_000));
        message.upsert_into(&*db.connection.lock().await)?;
        message = message.timestamp(Local.timestamp_millis(1_566_424_128_000));
        message.upsert_into(&*db.connection.lock().await)?;
        assert_eq!(db.select_sensor("test").await?, Some(message.into()));
        Ok(())
    }

    #[async_std::test]
    async fn select_actuals_ok() -> Result {
        let message = Message::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000));
        let db = Connection::open_and_initialize(":memory:").await?;
        message.upsert_into(&*db.connection.lock().await)?;
        assert_eq!(db.select_actuals().await?, vec![(message.sensor, message.reading)]);
        Ok(())
    }

    #[async_std::test]
    async fn existing_sensor_is_reused() -> Result {
        let db = Connection::open_and_initialize(":memory:").await?;
        let old = Message::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000));
        old.upsert_into(&*db.connection.lock().await)?;
        let new = Message::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_129_000));
        new.upsert_into(&*db.connection.lock().await)?;

        assert_eq!(db.select_sensor_count().await?, 1);

        Ok(())
    }

    #[async_std::test]
    async fn select_readings_ok() -> Result {
        let db = Connection::open_and_initialize(":memory:").await?;
        let message = Message::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000));
        message.upsert_into(&*db.connection.lock().await)?;
        let readings = db.select_readings("test", &Local.timestamp_millis(0)).await?;
        assert_eq!(readings.get(0).unwrap(), &message.reading);
        Ok(())
    }

    #[async_std::test]
    async fn get_set_user_data_ok() -> Result {
        let db = Connection::open_and_initialize(":memory:").await?;
        db.set_user_data("hello::world", 42_i32, Some(Local::now() + Duration::minutes(1)))
            .await?;
        assert_eq!(db.get_user_data("hello::world").await?, Some(42_i32));
        Ok(())
    }

    #[async_std::test]
    async fn get_set_user_data_overwrite_ok() -> Result {
        let db = Connection::open_and_initialize(":memory:").await?;
        db.set_user_data("hello::world", 43_i32, None).await?;
        db.set_user_data("hello::world", 42_i32, None).await?;
        assert_eq!(db.get_user_data("hello::world").await?, Some(42_i32));
        Ok(())
    }

    #[async_std::test]
    async fn get_expired_user_data_ok() -> Result {
        let db = Connection::open_and_initialize(":memory:").await?;
        db.set_user_data("hello::world", 43_i32, Some(Local::now() - Duration::minutes(1)))
            .await?;
        assert_eq!(db.get_user_data::<i32>("hello::world").await?, None);
        Ok(())
    }

    #[async_std::test]
    async fn missing_user_data_returns_none() -> Result {
        let db = Connection::open_and_initialize(":memory:").await?;
        assert_eq!(db.get_user_data::<String>("hello::world").await?, None);
        Ok(())
    }
}
