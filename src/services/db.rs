use crate::reading::{Message, Reading, Type};
use crate::services::Service;
use crate::threading;
use crate::value::Value;
use crate::Result;
use chrono::Local;
use multiqueue::{BroadcastReceiver, BroadcastSender};
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct Db {
    service_id: String,
    interval: Duration,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    /// Interval in milliseconds.
    pub interval_ms: Option<u64>,
}

impl Db {
    pub fn new(service_id: &str, settings: &Settings) -> Db {
        Db {
            service_id: service_id.into(),
            interval: Duration::from_millis(settings.interval_ms.unwrap_or(1000)),
        }
    }
}

impl Service for Db {
    fn spawn(
        self: Box<Self>,
        db: Arc<Mutex<crate::db::Db>>,
        tx: &BroadcastSender<Message>,
        _rx: &BroadcastReceiver<Message>,
    ) -> Result<()> {
        let tx = tx.clone();
        let sensor = format!("{}::size", &self.service_id);

        threading::spawn(format!("my-iot::db:{}", &self.service_id), move || loop {
            let size = { db.lock().unwrap().select_size().unwrap() };

            tx.try_send(Message {
                type_: Type::Actual,
                reading: Reading {
                    sensor: sensor.clone(),
                    value: Value::Size(size),
                    timestamp: Local::now(),
                },
            })
            .unwrap();

            thread::sleep(self.interval);
        })?;

        Ok(())
    }
}
