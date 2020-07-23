use crate::prelude::*;

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
        let tx = bus.add_tx();
        let rx = bus.add_rx();

        thread::Builder::new()
            .name(service_id.clone())
            .spawn(move || -> Result<(), ()> {
                let mut state = None;
                for message in &rx {
                    if message.sensor.id != self.sensor_id {
                        continue;
                    }
                    let value = if let Ok(value) = TryInto::<f64>::try_into(&message.reading.value) {
                        value
                    } else {
                        error!("[{}] Value is not `f64`.", &service_id);
                        continue;
                    };
                    if state == Some(State::Low) && value >= self.high {
                        state = Some(State::High);
                        Self::send_message(&service_id, "high", message, &tx);
                    } else if state == Some(State::High) && value < self.low {
                        state = Some(State::Low);
                        Self::send_message(&service_id, "low", message, &tx);
                    }
                }

                unreachable!();
            })?;

        Ok(())
    }

    /// Sends a message with the sensor ID of `<service_id>::<original_sensor_id>::<low|high>`.
    fn send_message(service_id: &str, suffix: &str, base_message: Message, tx: &Sender) {
        Message::new(format!("{}::{}::{}", service_id, &base_message.sensor.id, suffix))
            .type_(MessageType::ReadNonLogged)
            .value(base_message.reading.value)
            .timestamp(base_message.reading.timestamp)
            .optional_sensor_title(base_message.sensor.title)
            .location(base_message.sensor.location)
            .send_and_forget(tx);
    }
}
