//! Database interface.

use chrono::prelude::*;
use sqlx::sqlite::{SqliteConnectOptions, SqliteDone, SqliteJournalMode, SqliteRow};
use sqlx::{query, query_scalar, Row, SqliteConnection, SqlitePool};

use crate::prelude::*;

pub mod migrations;
pub mod reading;
pub mod sensor;
pub mod tasks;

/// Wraps the connection and provides the high-level database methods.
#[derive(Clone)]
pub struct Connection {
    inner: SqlitePool,
}

impl Connection {
    pub async fn open(uri: &str) -> Result<Self> {
        let connection = Self {
            inner: SqlitePool::connect_with(
                SqliteConnectOptions::new()
                    .filename(uri)
                    .create_if_missing(true)
                    .journal_mode(SqliteJournalMode::Wal)
                    .foreign_keys(false),
            )
            .await?,
        };
        connection.migrate().await?;
        Ok(connection)
    }

    async fn migrate(&self) -> Result {
        let user_version = self.get_user_version().await?;
        for (i, migration) in migrations::MIGRATIONS.iter().enumerate() {
            let i = i as i32;
            if user_version < i + 1 {
                info!("Applying migration #{}…", i + 1);
                {
                    let mut transaction = self.inner.begin().await?;
                    query(migration)
                        .execute_many(&mut transaction)
                        .await
                        .try_collect::<SqliteDone>()
                        .await?;
                    transaction.commit().await?;
                }

                info!("Vacuuming…");
                query("VACUUM").execute(&self.inner).await?;
            }
        }
        Ok(())
    }

    pub async fn get_user_version(&self) -> Result<i32> {
        // TODO: `fetch_one` issue: https://github.com/launchbadge/sqlx/issues/662
        // language=sql
        Ok(*query_scalar("PRAGMA user_version")
            .fetch_all(&self.inner)
            .await?
            .first()
            .unwrap())
    }

    async fn upsert_message_to(connection: &mut SqliteConnection, message: &Message) -> Result {
        let sensor_pk = hash_sensor_id(&message.sensor.id);
        let timestamp = message.reading.timestamp.timestamp_millis();
        let value = bincode::serialize(&message.reading.value)?;

        query(
            // language=sql
            r#"
                -- noinspection SqlResolve @ any/"excluded"
                REPLACE INTO sensors (pk, sensor_id, title, timestamp, location, value, is_writable)
                VALUES (?, ?, ?, ?, ?, ?, ?);

                -- noinspection SqlResolve @ any/"excluded"
                REPLACE INTO readings (sensor_fk, timestamp, value)
                VALUES (?, ?, ?);
            "#,
        )
        .bind(sensor_pk)
        .bind(&message.sensor.id)
        .bind(&message.sensor.title)
        .bind(timestamp)
        .bind(&message.sensor.location)
        .bind(&value)
        .bind(message.sensor.is_writable)
        .bind(sensor_pk)
        .bind(timestamp)
        .bind(&value)
        .execute(connection)
        .await?;

        Ok(())
    }

    #[cfg(test)]
    pub async fn upsert_message(&self, message: &Message) -> Result {
        let mut transaction = self.inner.begin().await?;
        Self::upsert_message_to(&mut transaction, &message).await?;
        transaction.commit().await?;
        Ok(())
    }

    /// Upserts the messages within a single transaction.
    ///
    /// Inserting messages one by one is quite slow on low-performance boards.
    /// Thus, I spin up a separate thread which accumulates incoming messages
    /// and periodically upserts them all within a single transaction.
    pub async fn upsert_messages(&self, messages: Vec<Message>) -> Result {
        let mut transaction = self.inner.begin().await?;

        for message in messages.iter() {
            debug!("[{:?}] {}", &message.type_, &message.sensor.id);
            Connection::upsert_message_to(&mut transaction, &message).await?;
        }
        transaction.commit().await?;

        Ok(())
    }

    /// Selects the latest readings for all sensors.
    pub async fn select_actuals(&self) -> Result<Vec<(Sensor, Reading)>> {
        // language=sql
        Ok(query(r"SELECT * FROM sensors ORDER BY location, sensor_id")
            .try_map(get_sensor_reading)
            .fetch_all(&self.inner)
            .await?)
    }

    /// Selects the database size.
    pub async fn select_size(&self) -> Result<i64> {
        // language=sql
        const QUERY: &str = r#"
            -- noinspection SqlResolve
            SELECT page_count * page_size as size FROM pragma_page_count(), pragma_page_size()
        "#;
        Ok(*query_scalar(QUERY).fetch_all(&self.inner).await?.first().unwrap())
    }

    /// Selects the specified sensor.
    pub async fn select_sensor(&self, sensor_id: &str) -> Result<Option<(Sensor, Reading)>> {
        // language=sql
        Ok(query(r"SELECT * FROM sensors WHERE sensor_id = ?")
            .bind(sensor_id)
            .try_map(get_sensor_reading)
            .fetch_optional(&self.inner)
            .await?)
    }

    pub async fn delete_sensor(&self, sensor_id: &str) -> Result {
        // language=sql
        query(r"DELETE FROM sensors WHERE sensor_id = ?")
            .bind(sensor_id)
            .execute(&self.inner)
            .await?;
        Ok(())
    }

    /// Selects the specified sensor readings within the specified period.
    pub async fn select_readings(&self, sensor_id: &str, since: &DateTime<Local>) -> Result<Vec<Reading>> {
        // language=sql
        const QUERY: &str = r#"
            SELECT timestamp, value
            FROM readings
            WHERE sensor_fk = ? AND timestamp >= ?
            ORDER BY timestamp
        "#;
        Ok(query(QUERY)
            .bind(hash_sensor_id(sensor_id))
            .bind(since.timestamp_millis())
            .try_map(get_reading)
            .fetch_all(&self.inner)
            .await?)
    }

    pub async fn select_last_n_readings(&self, sensor_id: &str, limit: i64) -> Result<Vec<Reading>> {
        // language=sql
        const QUERY: &str = "SELECT timestamp, value FROM readings WHERE sensor_fk = ? ORDER BY timestamp LIMIT ?";
        Ok(query(QUERY)
            .bind(hash_sensor_id(sensor_id))
            .bind(limit)
            .try_map(get_reading)
            .fetch_all(&self.inner)
            .await?)
    }

    pub async fn select_sensor_count(&self) -> Result<i64> {
        // language=sql
        Ok(*query_scalar("SELECT COUNT(*) FROM sensors")
            .fetch_all(&self.inner)
            .await?
            .first()
            .unwrap())
    }

    pub async fn select_total_reading_count(&self) -> Result<i64> {
        // language=sql
        Ok(*query_scalar("SELECT COUNT(*) FROM readings")
            .fetch_all(&self.inner)
            .await?
            .first()
            .unwrap())
    }

    pub async fn select_sensor_reading_count(&self, sensor_id: &str) -> Result<i64> {
        // language=sql
        Ok(*query_scalar("SELECT COUNT(*) FROM readings WHERE sensor_fk = ?")
            .bind(hash_sensor_id(sensor_id))
            .fetch_all(&self.inner)
            .await?
            .first()
            .unwrap())
    }

    pub async fn set_user_data<V: Serialize>(
        &self,
        key: &str,
        value: V,
        expires_at: Option<DateTime<Local>>,
    ) -> Result {
        // language=sql
        const QUERY: &str = r#"
            -- noinspection SqlResolve @ any/"excluded"
            REPLACE INTO user_data (pk, value, expires_at) VALUES (?, ?, ?);
        "#;
        query(QUERY)
            .bind(key)
            .bind(bincode::serialize(&value)?)
            .bind(expires_at.as_ref().map(DateTime::<Local>::timestamp_millis))
            .execute(&self.inner)
            .await?;
        Ok(())
    }

    pub async fn get_user_data<V: DeserializeOwned + Send + Unpin>(&self, key: &str) -> Result<Option<V>> {
        // language=sql
        const QUERY: &str = r#"
            SELECT value FROM user_data
            WHERE pk = ? AND (expires_at IS NULL OR expires_at >= ?)
        "#;
        Ok(query(QUERY)
            .bind(key)
            .bind(Local::now().timestamp_millis())
            .try_map(|row: SqliteRow| Ok(bincode::deserialize(&row.try_get::<Vec<u8>, _>(0)?).unwrap()))
            .fetch_optional(&self.inner)
            .await?)
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
fn get_sensor(row: &SqliteRow) -> StdResult<Sensor, sqlx::Error> {
    Ok(Sensor {
        id: row.try_get("sensor_id")?,
        title: row.try_get("title")?,
        location: row.try_get("location")?,
        is_writable: row.try_get("is_writable")?,
    })
}

/// Builds a `Reading` instance based on the database row.
fn get_reading<R: Borrow<SqliteRow>>(row: R) -> StdResult<Reading, sqlx::Error> {
    let row = row.borrow();
    Ok(Reading {
        timestamp: Local.timestamp_millis(row.try_get("timestamp")?),
        value: bincode::deserialize(&row.try_get::<Vec<u8>, _>("value")?).unwrap_or(Value::Other),
    })
}

fn get_sensor_reading<R: Borrow<SqliteRow>>(row: R) -> StdResult<(Sensor, Reading), sqlx::Error> {
    let row = row.borrow();
    Ok((get_sensor(row)?, get_reading(row)?))
}

impl From<Message> for (Sensor, Reading) {
    fn from(message: Message) -> Self {
        (message.sensor, message.reading)
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;

    #[async_std::test]
    async fn double_upsert_keeps_one_reading() -> Result {
        let message = Message::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000));

        let db = Connection::open(":memory:").await?;
        db.upsert_message(&message).await?;
        db.upsert_message(&message).await?;

        assert_eq!(db.select_total_reading_count().await?, 1);

        Ok(())
    }

    #[async_std::test]
    async fn select_last_reading_returns_none_on_empty_database() -> Result {
        let db = Connection::open(":memory:").await?;
        assert_eq!(db.select_sensor("test").await?, None);
        Ok(())
    }

    #[async_std::test]
    async fn select_last_reading_ok() -> Result {
        let message = Message::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000));
        let db = Connection::open(":memory:").await?;
        db.upsert_message(&message).await?;
        assert_eq!(db.select_sensor("test").await?, Some(message.into()));
        Ok(())
    }

    #[async_std::test]
    async fn insert_two_readings_ok() -> Result {
        let db = Connection::open(":memory:").await?;
        let mut message = Message::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_127_000));
        db.upsert_message(&message).await?;
        message = message.timestamp(Local.timestamp_millis(1_566_424_128_000));
        db.upsert_message(&message).await?;

        assert_eq!(db.select_total_reading_count().await?, 2);
        assert_eq!(db.select_sensor("test").await?, Some(message.into()));

        Ok(())
    }

    #[async_std::test]
    async fn select_actuals_ok() -> Result {
        let message = Message::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000));
        let db = Connection::open(":memory:").await?;
        db.upsert_message(&message).await?;
        assert_eq!(db.select_actuals().await?, vec![(message.sensor, message.reading)]);
        Ok(())
    }

    #[async_std::test]
    async fn existing_sensor_is_reused() -> Result {
        let db = Connection::open(":memory:").await?;
        let old = Message::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000));
        db.upsert_message(&old).await?;
        let new = Message::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_129_000));
        db.upsert_message(&new).await?;

        assert_eq!(db.select_sensor_count().await?, 1);

        Ok(())
    }

    #[async_std::test]
    async fn select_readings_ok() -> Result {
        let db = Connection::open(":memory:").await?;
        let message = Message::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000));
        db.upsert_message(&message).await?;
        let readings = db.select_readings("test", &Local.timestamp_millis(0)).await?;
        assert_eq!(readings.get(0).unwrap(), &message.reading);
        Ok(())
    }

    #[async_std::test]
    async fn get_set_user_data_ok() -> Result {
        let db = Connection::open(":memory:").await?;
        db.set_user_data("hello::world", 42_i32, Some(Local::now() + Duration::minutes(1)))
            .await?;
        assert_eq!(db.get_user_data("hello::world").await?, Some(42_i32));
        Ok(())
    }

    #[async_std::test]
    async fn get_set_user_data_overwrite_ok() -> Result {
        let db = Connection::open(":memory:").await?;
        db.set_user_data("hello::world", 43_i32, None).await?;
        db.set_user_data("hello::world", 42_i32, None).await?;
        assert_eq!(db.get_user_data("hello::world").await?, Some(42_i32));
        Ok(())
    }

    #[async_std::test]
    async fn get_expired_user_data_ok() -> Result {
        let db = Connection::open(":memory:").await?;
        db.set_user_data("hello::world", 43_i32, Some(Local::now() - Duration::minutes(1)))
            .await?;
        assert_eq!(db.get_user_data::<i32>("hello::world").await?, None);
        Ok(())
    }

    #[async_std::test]
    async fn missing_user_data_returns_none() -> Result {
        let db = Connection::open(":memory:").await?;
        assert_eq!(db.get_user_data::<String>("hello::world").await?, None);
        Ok(())
    }
}
