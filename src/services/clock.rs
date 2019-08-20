use crate::db::Db;
use crate::reading::*;
use crate::threading;
use crate::threading::JoinHandle;
use crate::value::Value;
use chrono::Local;
use crossbeam_channel::{Receiver, Sender};
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct Clock {
    service_id: String,
    interval: Duration,
    counter: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    /// Interval in milliseconds.
    pub interval_ms: Option<u64>,
}

impl Clock {
    pub fn new(service_id: &str, settings: &Settings) -> Clock {
        Clock {
            service_id: service_id.into(),
            interval: Duration::from_millis(settings.interval_ms.unwrap_or(1000)),
            counter: 0,
        }
    }
}

impl crate::services::Service for Clock {
    fn spawn(mut self: Box<Self>, _db: Arc<Mutex<Db>>, tx: Sender<Reading>, _rx: Receiver<Reading>) -> Vec<JoinHandle> {
        vec![threading::spawn(self.service_id.clone(), move || loop {
            #[rustfmt::skip]
            tx.send(Reading {
                sensor: self.service_id.clone(),
                value: Value::Counter(self.counter),
                timestamp: Local::now(),
                is_persisted: true,
            }).unwrap();

            self.counter += 1;
            thread::sleep(self.interval);
        })]
    }
}
