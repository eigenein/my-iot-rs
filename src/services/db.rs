use crate::prelude::*;
use std::thread;
use std::time::Duration;

const INTERVAL: Duration = Duration::from_secs(60);

pub struct Db;

impl Db {
    pub fn spawn(self, service_id: String, bus: &mut Bus, db: Connection) -> Result<()> {
        let tx = bus.add_tx();

        thread::Builder::new().name(service_id).spawn(move || loop {
            if let Err(error) = self.loop_(&db, &tx) {
                error!("Failed to refresh the sensors: {}", error.to_string());
            }
            thread::sleep(INTERVAL);
        })?;

        Ok(())
    }

    fn loop_(&self, db: &Connection, tx: &Sender) -> Result<()> {
        let expires_at = Local::now() + chrono::Duration::seconds(120);
        tx.send(
            Message::new("db::size")
                .value(Value::DataSize(db.select_size()?))
                .sensor_title("Database Size".to_string())
                .room_title("System".to_string())
                .expires_at(expires_at),
        )?;
        tx.send(
            Message::new("db::sensor_count")
                .value(Value::Counter(db.select_sensor_count()?))
                .sensor_title("Sensor Count")
                .room_title("System".to_string())
                .expires_at(expires_at),
        )?;
        tx.send(
            Message::new("db::reading_count")
                .value(Value::Counter(db.select_reading_count()?))
                .sensor_title("Reading Count")
                .room_title("System".to_string())
                .expires_at(expires_at),
        )?;
        Ok(())
    }
}
