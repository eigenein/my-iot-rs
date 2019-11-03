//! Database interface.

use crate::prelude::*;
use crate::supervisor;
use chrono::prelude::*;
use crossbeam_channel::Sender;
use log::{debug, info};
use rusqlite::{params, Connection, Row};
use std::path::Path;
use std::sync::{Arc, Mutex};

mod primitives;
mod value;

// FIXME: find a way to dial with `rusqlite::Error`.

const SCHEMA: &str = "
    CREATE TABLE IF NOT EXISTS sensors (
        id INTEGER PRIMARY KEY,
        sensor TEXT UNIQUE NOT NULL,
        type INTEGER NOT NULL,
        last_reading_id INTEGER NULL REFERENCES readings(id) ON UPDATE CASCADE ON DELETE CASCADE
    );

    -- Stores all sensor readings.
    CREATE TABLE IF NOT EXISTS readings (
        id INTEGER PRIMARY KEY,
        sensor_id INTEGER REFERENCES sensors(id) ON UPDATE CASCADE ON DELETE CASCADE,
        timestamp INTEGER NOT NULL,
        value BLOB NOT NULL
    );
    -- Descending index on `timestamp` is needed to speed up the select last queries.
    CREATE UNIQUE INDEX IF NOT EXISTS readings_sensor_id_timestamp ON readings (sensor_id, timestamp DESC);

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
    /// Insert or replace reading into database.
    pub fn upsert_reading(&self, message: &Message) -> Result<()> {
        let (type_, value) = message.value.serialize();
        self.connection
            .prepare_cached("INSERT OR IGNORE INTO sensors (sensor, type) VALUES (?1, ?2)")
            .unwrap()
            .execute(params![message.sensor, type_])?;
        let sensor_id = self.connection.last_insert_rowid();
        self.connection
            .prepare_cached("INSERT OR REPLACE INTO readings (sensor_id, timestamp, value) VALUES (?1, ?2, ?3)")
            .unwrap()
            .execute(params![sensor_id, message.timestamp.timestamp_millis(), value])?;
        let reading_id = self.connection.last_insert_rowid();
        self.connection
            .prepare_cached("UPDATE sensors SET last_reading_id = ?1 WHERE id = ?2")
            .unwrap()
            .execute(params![reading_id, sensor_id])?;
        Ok(())
    }

    /// Select latest reading for each sensor.
    pub fn select_actuals(&self) -> Result<Vec<Message>> {
        Ok(self
            .connection
            .prepare_cached(
                r#"
                SELECT sensor, timestamp, type, value
                FROM sensors
                INNER JOIN readings ON readings.id = sensors.last_reading_id
                "#,
            )?
            .query_map(params![], |row| Ok(Message::from_row(row).unwrap()))
            .unwrap()
            .map(rusqlite::Result::unwrap)
            .collect())
    }

    /// Select database size in bytes.
    pub fn select_size(&self) -> Result<u64> {
        Ok(self
            .connection
            .prepare_cached("SELECT page_count * page_size as size FROM pragma_page_count(), pragma_page_size()")?
            .query_row(params![], |row| row.get::<_, i64>(0))
            .map(|v| v as u64)
            .unwrap())
    }

    /// Select the very last sensor reading.
    pub fn select_last_reading(&self, sensor: &str) -> Result<Option<Message>> {
        Ok(self
            .connection
            .prepare_cached(
                r#"
                SELECT sensor, timestamp, type, value
                FROM sensors
                INNER JOIN readings ON readings.id = sensors.last_reading_id
                WHERE sensors.sensor = ?1
                "#,
            )
            .unwrap()
            .query_row(params![sensor], |row| Ok(Some(Message::from_row(row).unwrap())))
            .unwrap_or(None))
    }

    /// Select the latest sensor readings within the given time interval.
    pub fn select_readings(&self, sensor: &str, since: &DateTime<Local>) -> Result<Vec<Message>> {
        Ok(self
            .connection
            .prepare_cached(
                r#"
                SELECT sensor, type, timestamp, value
                FROM readings
                INNER JOIN sensors ON sensors.id = readings.sensor_id
                WHERE sensors.sensor = ?1 AND timestamp >= ?2
                ORDER BY timestamp
                "#,
            )
            .unwrap()
            .query_map(params![sensor, since.timestamp_millis()], |row| {
                Ok(Message::from_row(row).unwrap())
            })
            .unwrap()
            .map(rusqlite::Result::unwrap)
            .collect())
    }
}

// TODO: make `Reading` struct.
impl Message {
    fn from_row(row: &Row) -> Result<Message> {
        Ok(Message {
            type_: MessageType::ReadLogged,
            sensor: row.get_unwrap("sensor"),
            timestamp: Local.timestamp_millis(row.get_unwrap("timestamp")),
            value: Value::deserialize(row.get_unwrap("type"), row.get_unwrap("value"))?,
        })
    }
}

/// Spawn the persistence thread.
pub fn spawn(db: Arc<Mutex<Db>>, tx: &Sender<Message>) -> Result<Sender<Message>> {
    info!("Spawning readings persistence…");
    let tx = tx.clone();
    let (out_tx, rx) = crossbeam_channel::unbounded::<Message>();

    supervisor::spawn("my-iot::persistence", tx.clone(), move || {
        for message in &rx {
            process_message(message, &db, &tx).unwrap();
        }
        unreachable!();
    })?;

    Ok(out_tx)
}

/// Process a message.
fn process_message(message: Message, db: &Arc<Mutex<Db>>, tx: &Sender<Message>) -> Result<()> {
    info!("{}: {:?} {:?}", &message.sensor, &message.type_, &message.value,);
    debug!("{:?}", &message);
    // TODO: handle `ReadSnapshot`.
    if message.type_ == MessageType::ReadLogged {
        let db = db.lock().unwrap();
        let previous_reading = db.select_last_reading(&message.sensor)?;
        db.upsert_reading(&message)?;
        send_messages(&previous_reading, &message, &tx)?;
    }
    Ok(())
}

/// Check if sensor value has been updated or changed and send corresponding messages.
fn send_messages(previous_reading: &Option<Message>, message: &Message, tx: &Sender<Message>) -> Result<()> {
    if let Some(existing) = previous_reading {
        if message.timestamp > existing.timestamp {
            tx.send(
                Composer::new(format!("{}::update", &message.sensor))
                    .type_(MessageType::ReadNonLogged)
                    .value(message.value.clone())
                    .into(),
            )?;
            if message.value != existing.value {
                tx.send(
                    Composer::new(format!("{}::change", &message.sensor))
                        .type_(MessageType::ReadNonLogged)
                        .value(message.value.clone())
                        .into(),
                )?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    type Result = crate::Result<()>;

    #[test]
    fn double_upsert_keeps_one_record() -> Result {
        let reading = Composer::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000))
            .into();

        let db = Db::new(":memory:")?;
        db.upsert_reading(&reading)?;
        db.upsert_reading(&reading)?;

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
    fn select_last_reading_ok() -> Result {
        let reading = Composer::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000))
            .into();
        let db = Db::new(":memory:")?;
        db.upsert_reading(&reading)?;
        assert_eq!(db.select_last_reading("test")?, Some(reading));
        Ok(())
    }

    #[test]
    fn select_last_reading_returns_newer_reading() -> Result {
        let db = Db::new(":memory:")?;
        db.upsert_reading(
            &Composer::new("test")
                .value(Value::Counter(42))
                .timestamp(Local.timestamp_millis(1_566_424_127_000))
                .into(),
        )?;
        let new = Composer::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000))
            .into();
        db.upsert_reading(&new)?;
        assert_eq!(db.select_last_reading("test")?, Some(new));
        Ok(())
    }

    #[test]
    fn select_actuals_ok() -> Result {
        let message = Composer::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000))
            .into();
        let db = Db::new(":memory:")?;
        db.upsert_reading(&message)?;
        assert_eq!(db.select_actuals()?, vec![message]);
        Ok(())
    }

    #[test]
    fn existing_sensor_is_reused() -> Result {
        let db = Db::new(":memory:")?;
        db.upsert_reading(
            &Composer::new("test")
                .value(Value::Counter(42))
                .timestamp(Local.timestamp_millis(1_566_424_128_000))
                .into(),
        )?;
        db.upsert_reading(
            &Composer::new("test")
                .value(Value::Counter(42))
                .timestamp(Local.timestamp_millis(1_566_424_129_000))
                .into(),
        )?;

        let reading_count = db
            .connection
            .prepare("SELECT COUNT(*) FROM sensors")?
            .query_row(params![], |row| row.get::<_, i64>(0))?;
        assert_eq!(reading_count, 1);

        Ok(())
    }
}
