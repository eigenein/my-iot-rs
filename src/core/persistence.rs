//! Database interface.

use crate::core::persistence::schema::*;
use crate::prelude::*;
use crate::supervisor;
use chrono::prelude::*;
use crossbeam_channel::Sender;
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use log::{debug, info};
use std::sync::{Arc, Mutex};

pub mod models;
mod primitives;
mod schema;
mod thread;
mod value;

embed_migrations!("migrations");

pub fn establish_connection(url: &str) -> Result<SqliteConnection> {
    let db = SqliteConnection::establish(url)?;
    db.batch_execute(
        r#"
        PRAGMA synchronous = NORMAL;
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;
        "#,
    )?;
    embedded_migrations::run(&db);
    Ok(db)
}

pub fn upsert_message(db: &SqliteConnection, message: &Message) -> Result<()> {
    diesel::replace_into(sensors::table)
        .values(&message.sensor)
        .execute(db)?;
    let sensor_id: u64 = sensors
        .filter(sensors::dsl::sensor.eq(message.sensor.sensor))
        .select(sensors::dsl::id)?;
    Ok(())
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
        upsert_message(&db, &message)?;
        upsert_message(&db, &message)?;

        let reading_count = db
            .connection
            .prepare("SELECT COUNT(*) FROM readings")?
            .query_row(params![], |row| row.get::<_, i64>(0))?;
        assert_eq!(reading_count, 1);

        Ok(())
    }

    #[test]
    fn select_last_reading_returns_none_on_empty_database() -> Result {
        let db = establish_connection(":memory:")?;
        assert_eq!(db.select_last_reading("test")?, None);
        Ok(())
    }

    #[test]
    fn select_last_reading_ok() -> Result {
        let reading = Composer::new("test")
            .value(Value::Counter(42))
            .timestamp(Local.timestamp_millis(1_566_424_128_000))
            .into();
        let db = establish_connection(":memory:")?;
        db.upsert_reading(&reading)?;
        assert_eq!(db.select_last_reading("test")?, Some(reading));
        Ok(())
    }

    #[test]
    fn select_last_reading_returns_newer_reading() -> Result {
        let db = establish_connection(":memory:")?;
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
        let db = establish_connection(":memory:")?;
        db.upsert_reading(&message)?;
        assert_eq!(db.select_actuals()?, vec![message]);
        Ok(())
    }

    #[test]
    fn existing_sensor_is_reused() -> Result {
        let db = establish_connection(":memory:")?;
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
