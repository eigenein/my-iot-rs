use crate::reading::Reading;
use crate::services::Service;
use crate::threading;
use crate::value::Value;
use chrono::Local;
use crossbeam_channel::{Receiver, Sender};
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

#[derive(Debug)]
pub struct Db {
    interval: Duration,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DbSettings {
    /// Interval in milliseconds.
    pub interval_ms: Option<u64>,
}

impl Db {
    pub fn new(settings: &DbSettings) -> Db {
        Db {
            interval: Duration::from_millis(settings.interval_ms.unwrap_or(1000)),
        }
    }
}

impl Service for Db {
    fn spawn(
        self: Box<Self>,
        service_id: String,
        db: Arc<Mutex<crate::db::Db>>,
        tx: Sender<Reading>,
        _rx: Receiver<Reading>,
    ) -> Vec<JoinHandle<()>> {
        let sensor = format!("{}:size", &service_id);
        vec![threading::spawn(service_id, move || loop {
            let size = { db.lock().unwrap().select_size() };

            #[rustfmt::skip]
            tx.send(Reading {
                sensor: sensor.clone(),
                value: Value::Size(size),
                timestamp: Local::now(),
                is_persisted: true,
            }).unwrap();

            thread::sleep(self.interval);
        })]
    }
}
