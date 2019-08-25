use crate::reading::Reading;
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
        tx: &BroadcastSender<Reading>,
        _rx: &BroadcastReceiver<Reading>,
    ) -> Result<()> {
        let tx = tx.clone();
        let sensor = format!("{}::size", &self.service_id);

        threading::spawn(self.service_id.clone(), move || loop {
            let size = { db.lock().unwrap().select_size().unwrap() };

            tx.try_send(Reading {
                sensor: sensor.clone(),
                value: Value::Size(size),
                timestamp: Local::now(),
                is_persisted: true,
            })
            .unwrap();

            thread::sleep(self.interval);
        })?;

        Ok(())
    }
}
