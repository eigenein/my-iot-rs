use crate::event::Event;
use crate::services::Service;
use log::{debug, info};
use serde::Deserialize;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub struct Clock {
    interval: Duration,
}

/// Clock settings.
#[derive(Deserialize, Debug)]
pub struct ClockSettings {
    /// Interval in seconds.
    #[serde(default)]
    pub interval_ms: u64,
}

impl Default for ClockSettings {
    fn default() -> Self {
        ClockSettings { interval_ms: 1000 }
    }
}

impl Clock {
    pub fn new(settings: &ClockSettings) -> Clock {
        Clock {
            interval: Duration::from_millis(settings.interval_ms),
        }
    }
}

impl Service for Clock {
    fn run(&mut self, tx: Sender<Event>) {
        info!("Starting {:?}.", &self);
        loop {
            thread::sleep(self.interval);
            debug!("Clock event.");
        }
    }
}
