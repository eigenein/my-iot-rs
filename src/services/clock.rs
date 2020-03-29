use crate::prelude::*;
use crate::supervisor;
use serde::Deserialize;
use std::thread;
use std::time::Duration;

#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    /// Interval in milliseconds.
    #[serde(default = "default_interval_ms")]
    pub interval_ms: u64,
}

fn default_interval_ms() -> u64 {
    1000
}

pub fn spawn(service_id: &str, settings: &Settings, bus: &mut Bus) -> Result<()> {
    let service_id = service_id.to_string();
    let interval = Duration::from_millis(settings.interval_ms);
    let tx = bus.add_tx();

    supervisor::spawn(format!("my-iot::{}", service_id), tx.clone(), move || -> Result<()> {
        let mut counter = 1;
        loop {
            tx.send(Composer::new(&service_id).value(Value::Counter(counter)).into())?;
            counter += 1;
            thread::sleep(interval);
        }
    })?;

    Ok(())
}
