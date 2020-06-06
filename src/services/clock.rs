use crate::prelude::*;
use serde::Deserialize;
use std::thread;
use std::time::Duration;

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Clock {
    /// Interval in milliseconds.
    #[serde(default = "default_interval_ms")]
    pub interval_ms: u32,
}

fn default_interval_ms() -> u32 {
    1000
}

impl Clock {
    pub fn spawn(self, service_id: String, bus: &mut Bus) -> Result<()> {
        let interval = Duration::from_millis(self.interval_ms as u64);
        let ttl = chrono::Duration::milliseconds(self.interval_ms as i64);
        let tx = bus.add_tx();

        thread::Builder::new().name(service_id.clone()).spawn(move || {
            let mut counter = 1;
            loop {
                Message::new(&service_id)
                    .value(Value::Counter(counter))
                    .expires_in(ttl)
                    .send_and_forget(&tx);
                counter += 1;
                thread::sleep(interval);
            }
        })?;

        Ok(())
    }
}
