use crate::prelude::*;
use crate::supervisor;
use serde::Deserialize;
use std::thread;
use std::time::Duration;

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Clock {
    /// Interval in milliseconds.
    #[serde(default = "default_interval_ms")]
    pub interval_ms: u64,
}

fn default_interval_ms() -> u64 {
    1000
}

impl Clock {
    pub fn spawn<'env>(&self, scope: &Scope<'env>, service_id: &'env str, bus: &mut Bus) -> Result<()> {
        let interval = Duration::from_millis(self.interval_ms);
        let tx = bus.add_tx();

        supervisor::spawn(scope, service_id, tx.clone(), move || -> Result<()> {
            let mut counter = 1;
            loop {
                tx.send(Message::new(service_id).value(Value::Counter(counter)))?;
                counter += 1;
                thread::sleep(interval);
            }
        })?;

        Ok(())
    }
}
