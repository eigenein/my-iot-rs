use crate::prelude::*;
use crate::supervisor;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const INTERVAL: Duration = Duration::from_secs(60);

pub struct Db;

impl Service for Db {
    fn spawn(&self, service_id: &str, db: &Arc<Mutex<Connection>>, bus: &mut Bus) -> Result<()> {
        let tx = bus.add_tx();
        let db = db.clone();

        supervisor::spawn(service_id, tx.clone(), move || -> Result<()> {
            loop {
                {
                    let db = db.lock().unwrap();
                    let size = db.select_size()?;
                    tx.send(
                        Message::new("db::size")
                            .value(Value::DataSize(size))
                            .sensor_title("Database Size".to_string())
                            .room_title("System".to_string()),
                    )?;
                    tx.send(
                        Message::new("db::sensor_count")
                            .value(Value::Counter(db.select_sensor_count()?))
                            .sensor_title("Sensor Count")
                            .room_title("System".to_string()),
                    )?;
                    tx.send(
                        Message::new("db::reading_count")
                            .value(Value::Counter(db.select_reading_count()?))
                            .sensor_title("Reading Count")
                            .room_title("System".to_string()),
                    )?;
                }
                thread::sleep(INTERVAL);
            }
        })?;

        Ok(())
    }
}
