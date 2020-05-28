use crate::prelude::*;
use crate::supervisor;
use std::thread;
use std::time::Duration;

const INTERVAL: Duration = Duration::from_secs(60);

pub struct Db;

impl Db {
    pub fn spawn<'env>(
        &self,
        scope: &Scope<'env>,
        service_id: &'env str,
        bus: &mut Bus,
        db: Arc<Mutex<Connection>>,
    ) -> Result<()> {
        let tx = bus.add_tx();

        supervisor::spawn(scope, service_id, tx.clone(), move || -> Result<()> {
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
