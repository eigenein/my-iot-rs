//! Periodically emits messages.

use crate::prelude::*;
use serde::Deserialize;
use std::thread;
use std::time::Duration;

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Clock {
    /// Interval in milliseconds.
    #[serde(default = "default_interval_millis")]
    interval_millis: u64,
}

const fn default_interval_millis() -> u64 {
    1000
}

impl Clock {
    pub fn spawn(self, service_id: String, bus: &mut Bus) -> Result {
        let interval = Duration::from_millis(self.interval_millis);
        let tx = bus.add_tx();

        thread::spawn(move || {
            let mut counter = 1;
            loop {
                Message::new(&service_id)
                    .value(Value::Counter(counter))
                    .send_and_forget(&tx);
                counter += 1;
                thread::sleep(interval);
            }
        });

        Ok(())
    }
}
