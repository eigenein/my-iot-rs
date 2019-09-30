use crate::message::{Message, Reading, Type};
use crate::supervisor;
use crate::value::Value;
use crate::Result;
use chrono::Local;
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
    db: &Arc<Mutex<crate::db::Db>>,
    tx: &Sender<Message>,
) -> Result<Vec<Sender<Message>>> {
    let interval = Duration::from_millis(settings.interval_ms);
    let tx = tx.clone();
    let sensor = format!("{}::size", service_id);
    let db = db.clone();

    supervisor::spawn(format!("my-iot::db:{}", service_id), move || loop {
        let size = { db.lock().unwrap().select_size().unwrap() };

        tx.send(Message {
            type_: Type::Actual,
            reading: Reading {
                sensor: sensor.clone(),
                value: Value::Size(size),
                timestamp: Local::now(),
            },
        })
        .unwrap();

        thread::sleep(interval);
    })?;

    Ok(vec![])
}
