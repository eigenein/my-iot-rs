use crate::prelude::*;
use crate::services::prelude::*;

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Threshold {
    /// The monitored sensor ID, must be `f64`-compatible.
    sensor_id: String,

    low: f64,

    high: f64,
}

#[derive(PartialEq)]
enum State {
    Low,
    High,
}

impl Threshold {
    pub fn spawn(self, service_id: String, bus: &mut Bus) -> Result {
        let mut tx = bus.add_tx();
        let mut rx = bus.add_rx();

        task::spawn(async move {
            let mut state = None;
            while let Some(message) = rx.next().await {
                let value = match expect::<f64>(&service_id, &message, &self.sensor_id) {
                    Some(value) => value,
                    None => continue,
                };
                if (state == Some(State::Low) || state.is_none()) && value >= self.high {
                    state = Some(State::High);
                    Self::send_message(&service_id, "high", message, &mut tx).await;
                } else if (state == Some(State::High) || state.is_none()) && value < self.low {
                    state = Some(State::Low);
                    Self::send_message(&service_id, "low", message, &mut tx).await;
                }
            }
            unreachable!();
        });

        Ok(())
    }

    /// Sends a message with the sensor ID of `<service_id>::<original_sensor_id>::<low|high>`.
    async fn send_message(service_id: &str, suffix: &str, base_message: Message, tx: &mut Sender) {
        Message::new(format!("{}::{}::{}", service_id, &base_message.sensor.id, suffix))
            .type_(MessageType::ReadNonLogged)
            .value(base_message.reading.value)
            .timestamp(base_message.reading.timestamp)
            .optional_sensor_title(base_message.sensor.title)
            .location(base_message.sensor.location)
            .send_to(tx)
            .await;
    }
}
