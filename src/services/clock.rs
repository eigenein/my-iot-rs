use crate::db::Db;
use crate::message::*;
use crate::threading;
use crate::value::Value;
use crate::Result;
use bus::Bus;
use chrono::Local;
use crossbeam_channel::Sender;
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
    fn spawn(mut self: Box<Self>, _db: Arc<Mutex<Db>>, tx: &Sender<Message>, _rx: &mut Bus<Message>) -> Result<()> {
        let tx = tx.clone();

        threading::spawn(format!("my-iot::clock:{}", &self.service_id), move || loop {
            tx.try_send(Message {
                type_: Type::Actual,
                reading: Reading {
                    sensor: self.service_id.clone(),
                    value: Value::Counter(self.counter),
                    timestamp: Local::now(),
                },
            })
            .unwrap();

            self.counter += 1;
            thread::sleep(self.interval);
        })?;

        Ok(())
    }
}
