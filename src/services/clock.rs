//! Periodically emits messages.

use crate::prelude::*;

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
    pub async fn spawn(self, service_id: String, bus: &mut Bus) -> Result {
        let interval = Duration::from_millis(self.interval_millis);
        let mut tx = bus.add_tx();

        task::spawn(async move {
            let mut counter = 1;
            loop {
                Message::new(&service_id)
                    .value(Value::Counter(counter))
                    .send_to(&mut tx)
                    .await;
                counter += 1;
                task::sleep(interval).await;
            }
        });

        Ok(())
    }
}
