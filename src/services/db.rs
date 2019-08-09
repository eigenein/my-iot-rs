use crate::measurement::Measurement;
use crate::services::Service;
use crate::value::Value;
use serde::Deserialize;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
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
    fn run(&mut self, db: Arc<Mutex<crate::db::Db>>, tx: Sender<Measurement>) -> ! {
        loop {
            let size = { db.lock().unwrap().select_size() };

            #[rustfmt::skip]
            tx.send(Measurement::new(
                "db:size".to_string(),
                Value::Size(size),
                None,
            )).unwrap();

            thread::sleep(self.interval);
        }
    }
}
