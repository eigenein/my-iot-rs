use crate::prelude::*;
use crate::supervisor;
use crossbeam_channel::Sender;
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

pub fn spawn(
    service_id: &str,
    settings: &Settings,
    db: &Arc<Mutex<Connection>>,
    tx: &Sender<Message>,
) -> Result<Vec<Sender<Message>>> {
    let interval = Duration::from_millis(settings.interval_ms);
    let tx = tx.clone();
    let sensor = format!("{}::size", service_id);
    let db = db.clone();

    supervisor::spawn(
        format!("my-iot::db::{}", service_id),
        tx.clone(),
        move || -> Result<()> {
            loop {
                let size = { db.lock().unwrap().select_size().unwrap() };
                tx.send(Composer::new(sensor.clone()).value(Value::Size(size)).into())?;
                thread::sleep(interval);
            }
        },
    )?;

    Ok(vec![])
}
