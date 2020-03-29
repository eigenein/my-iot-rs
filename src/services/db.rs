use crate::core::persistence::*;
use crate::prelude::*;
use crate::supervisor;
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    /// Interval in milliseconds.
    #[serde(default = "default_interval_ms")]
    pub interval_ms: u64,
}

fn default_interval_ms() -> u64 {
    60000
}

pub fn spawn(service_id: &str, settings: &Settings, db: &Arc<Mutex<Connection>>, bus: &mut Bus) -> Result<()> {
    let interval = Duration::from_millis(settings.interval_ms);
    let tx = bus.add_tx();
    let sensor = format!("{}::size", service_id);
    let db = db.clone();

    supervisor::spawn(format!("my-iot::{}", service_id), tx.clone(), move || -> Result<()> {
        loop {
            let size = select_size(&db.lock().unwrap())?;
            tx.send(
                Composer::new(sensor.clone())
                    .value(Value::DataSize(size))
                    .title("Database Size".to_string())
                    .room_title("System".to_string())
                    .into(),
            )?;
            thread::sleep(interval);
        }
    })?;

    Ok(())
}
