use crate::db::Db;
use crate::measurement::*;
use crate::value::Value;
use serde::Deserialize;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Clock service.
#[derive(Debug)]
pub struct Clock {
    interval: Duration,
    sensor: String,
    counter: u64,
}

/// Clock settings.
#[derive(Deserialize, Debug)]
pub struct ClockSettings {
    /// Interval in milliseconds.
    pub interval_ms: Option<u64>,

    /// Sensor suffix.
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
    fn run(&mut self, _db: Arc<Mutex<Db>>, tx: Sender<Measurement>) {
        loop {
            #[rustfmt::skip]
            tx.send(Measurement::new(
                self.sensor.clone(),
                Value::Counter(self.counter),
                None,
            )).unwrap();

            self.counter += 1;
            thread::sleep(self.interval);
        }
    }
}
