use crate::measurement::*;
use crate::services::Service;
use crate::value::Value;
use log::info;
use serde::Deserialize;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

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

impl Service for Clock {
    fn run(&mut self, tx: Sender<Measurement>) {
        info!("Starting {:?}.", &self);
        loop {
            thread::sleep(self.interval);
            self.counter += 1;

            #[rustfmt::skip]
            tx.send(Measurement::new(
                self.sensor.clone(),
                Value::Counter(self.counter),
                None,
            )).unwrap();
        }
    }
}
