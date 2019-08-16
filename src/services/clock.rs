use crate::db::Db;
use crate::reading::*;
use crate::threading;
use crate::value::Value;
use chrono::Local;
use serde::Deserialize;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

#[derive(Debug)]
pub struct Clock {
    interval: Duration,
    sensor: String,
    counter: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ClockSettings {
    /// Interval in milliseconds.
    pub interval_ms: Option<u64>,

    /// Sensor suffix. Clock will yield readings under `clock:suffix` sensor.
    pub suffix: String,
}

impl Clock {
    pub fn new(settings: &ClockSettings) -> Clock {
        Clock {
            interval: Duration::from_millis(settings.interval_ms.unwrap_or(1000)),
            counter: 0,
            sensor: format!("clock:{}", settings.suffix),
        }
    }
}

impl crate::services::Service for Clock {
    fn spawn(mut self: Box<Self>, service_id: String, _db: Arc<Mutex<Db>>, tx: Sender<Reading>) -> Vec<JoinHandle<()>> {
        vec![threading::spawn(service_id, move || loop {
            #[rustfmt::skip]
            tx.send(Reading {
                sensor: self.sensor.clone(),
                value: Value::Counter(self.counter),
                timestamp: Local::now(),
                is_persisted: true,
            }).unwrap();

            self.counter += 1;
            thread::sleep(self.interval);
        })]
    }
}
