//! Database interface.

use crate::message::*;
use crate::value::Value;
use crate::Result;
use chrono::prelude::*;
use failure::format_err;
use rusqlite::{params, Connection, Row};
use std::convert::TryInto;
use std::path::Path;

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
        ts INTEGER NOT NULL,
        value BLOB NOT NULL
    );
    -- Descending index on `ts` is needed to speed up the select latest queries.
    CREATE UNIQUE INDEX IF NOT EXISTS readings_sensor_id_ts ON readings (sensor_id, ts DESC);

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
    /// Insert reading into database.
    pub fn insert_reading(&self, message: &Message) -> Result<()> {
        let (type_, value) = message.value.serialize();
        // TODO: handle `ReadSnapshot`.
        self.connection
            .prepare_cached("INSERT OR IGNORE INTO sensors (sensor, type) VALUES (?1, ?2)")
            .unwrap()
            .execute(params![message.sensor, type_])?;
        let sensor_id = self.connection.last_insert_rowid();
        self.connection
            .prepare_cached("INSERT OR REPLACE INTO readings (sensor_id, ts, value) VALUES (?1, ?2, ?3)")
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
    pub fn select_latest_readings(&self) -> Result<Vec<Message>> {
        Ok(self
            .connection
            .prepare_cached(
                r#"
                SELECT sensor, ts, type, value
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
                SELECT sensor, ts, type, value
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
                SELECT sensors.sensor, ts, value
                FROM readings
                INNER JOIN sensors ON sensors.id = readings.sensor_id
                WHERE sensors.sensor = ?1 AND ts >= ?2
                ORDER BY ts
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
            type_: Type::ReadLogged,
            sensor: row.get_unwrap("sensor"),
            timestamp: Local.timestamp_millis(row.get_unwrap("ts")),
            value: Value::deserialize(row.get_unwrap("type"), row.get_unwrap("value"))?,
        })
    }
}

// TODO: move to a separate module.
impl Value {
    fn serialize(&self) -> (u32, Vec<u8>) {
        match self {
            Value::None => (0, vec![]),
            Value::Boolean(value) => (if !value { 1 } else { 2 }, vec![]),
            Value::ImageUrl(value) => (3, value.as_bytes().to_vec()),
            Value::Text(value) => (4, value.as_bytes().to_vec()),
            Value::Bft(value) => (5, vec![*value]),
            Value::Celsius(value) => (6, value.to_bits().to_le_bytes().to_vec()),
            Value::Counter(value) => (7, value.to_le_bytes().to_vec()),
            Value::Metres(value) => (8, value.to_bits().to_le_bytes().to_vec()),
            Value::Rh(value) => (9, value.to_bits().to_le_bytes().to_vec()),
            Value::WindDirection(value) => (10, (*value as u32).to_le_bytes().to_vec()),
            Value::Size(value) => (11, value.to_le_bytes().to_vec()),
        }
    }

    fn deserialize(type_: u32, blob: Vec<u8>) -> Result<Self> {
        match type_ {
            0 => Ok(Value::None),
            1 => Ok(Value::Boolean(false)),
            2 => Ok(Value::Boolean(true)),
            3 => Ok(Value::ImageUrl(String::from_utf8(blob)?)),
            4 => Ok(Value::Text(String::from_utf8(blob)?)),
            5 => Ok(Value::Bft(blob[0])),
            6 => Ok(Value::Celsius(f64::from_bits(u64::from_le_bytes(
                (&blob[0..8]).try_into()?,
            )))),
            7 => Ok(Value::Counter(u64::from_le_bytes((&blob[0..8]).try_into()?))),
            // TODO: 8, 9, 10, 11
            _ => Err(format_err!("unknown value type: {}", type_)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Result = crate::Result<()>;

    #[test]
    fn reading_double_insert_keeps_one_record() -> Result {
        let reading = Composer::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000))
            .into();

        let db = Db::new(":memory:")?;
        db.insert_reading(&reading)?;
        db.insert_reading(&reading)?;

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
    fn select_last_reading_returns_test_reading() -> Result {
        let reading = Composer::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000))
            .into();
        let db = Db::new(":memory:")?;
        db.insert_reading(&reading)?;
        assert_eq!(db.select_last_reading("test")?, Some(reading));
        Ok(())
    }

    #[test]
    fn select_last_reading_returns_newer_reading() -> Result {
        let db = Db::new(":memory:")?;
        db.insert_reading(
            &Composer::new("test")
                .value(Value::Counter(42))
                .timestamp(Local.timestamp_millis(1_566_424_127_000))
                .into(),
        )?;
        let new = Composer::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000))
            .into();
        db.insert_reading(&new)?;
        assert_eq!(db.select_last_reading("test")?, Some(new));
        Ok(())
    }

    #[test]
    fn select_latest_readings_returns_test_reading() -> Result {
        let message = Composer::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000))
            .into();
        let db = Db::new(":memory:")?;
        db.insert_reading(&message)?;
        assert_eq!(db.select_latest_readings()?, vec![message]);
        Ok(())
    }

    #[test]
    fn existing_sensor_is_reused() -> Result {
        let db = Db::new(":memory:")?;
        db.insert_reading(
            &Composer::new("test")
                .value(Value::Counter(42))
                .timestamp(Local.timestamp_millis(1_566_424_128_000))
                .into(),
        )?;
        db.insert_reading(
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
