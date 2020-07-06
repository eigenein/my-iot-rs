//! System database service.

use crate::prelude::*;
use std::time::Duration;

const INTERVAL: Duration = Duration::from_secs(60);

pub struct Db;

impl Db {
    pub fn spawn(self, service_id: String, bus: &mut Bus, db: Connection) -> Result {
        let tx = bus.add_tx();
        spawn_service_loop(service_id, INTERVAL, move || self.loop_(&db, &tx))
    }

    fn loop_(&self, db: &Connection, tx: &Sender) -> Result {
        tx.send(
            Message::new("db::size")
                .value(Value::DataSize(db.select_size()?))
                .sensor_title("Database Size".to_string())
                .location("System"),
        )?;
        tx.send(
            Message::new("db::sensor_count")
                .value(Value::Counter(db.select_sensor_count()?))
                .sensor_title("Sensor Count")
                .location("System"),
        )?;
        tx.send(
            Message::new("db::reading_count")
                .value(Value::Counter(db.select_reading_count()?))
                .sensor_title("Reading Count")
                .location("System"),
        )?;
        Ok(())
    }
}
