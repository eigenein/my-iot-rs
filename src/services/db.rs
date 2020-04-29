use crate::prelude::*;
use crate::supervisor;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const INTERVAL: Duration = Duration::from_secs(60);

pub struct Db;

impl Service for Db {
    fn spawn(&self, _service_id: &str, db: &Arc<Mutex<Connection>>, bus: &mut Bus) -> Result<()> {
        let tx = bus.add_tx();
        let db = db.clone();

        supervisor::spawn("db", tx.clone(), move || -> Result<()> {
            loop {
                {
                    let db = db.lock().unwrap();
                    let size = db.select_size()?;
                    tx.send(
                        Composer::new("db::size")
                            .value(Value::DataSize(size))
                            .title("Database Size".to_string())
                            .room_title("System".to_string())
                            .into(),
                    )?;
                    tx.send(
                        Composer::new("db::sensor_count")
                            .value(Value::Counter(db.select_sensor_count()?))
                            .title("Sensor Count")
                            .room_title("System".to_string())
                            .into(),
                    )?;
                    tx.send(
                        Composer::new("db::reading_count")
                            .value(Value::Counter(db.select_reading_count()?))
                            .title("Reading Count")
                            .room_title("System".to_string())
                            .into(),
                    )?;
                }
                thread::sleep(INTERVAL);
            }
        })?;

        Ok(())
    }
}
