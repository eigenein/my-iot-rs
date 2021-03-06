use crate::prelude::*;
use crate::services::prelude::*;
use spa::{calc_sunrise_and_set, SunriseAndSet};

/// Emits durations to and after sunrise and sunset.
#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Solar {
    /// Message interval in milliseconds.
    #[serde(default = "default_interval_millis")]
    interval_millis: u64,

    /// Which location should the sensor be put into.
    #[serde(default)]
    location: Option<String>,

    secrets: Secrets,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Secrets {
    /// Latitude in [WGS84](https://en.wikipedia.org/wiki/World_Geodetic_System) system, ranging from `-90.0` to `90.0`.
    latitude: f64,

    /// Longitude in [WGS84](https://en.wikipedia.org/wiki/World_Geodetic_System) system, ranging from `-180.0` to `180.0`
    longitude: f64,
}

/// Defaults to one minute.
const fn default_interval_millis() -> u64 {
    60000
}

impl Solar {
    pub fn spawn(self, service_id: String, bus: &mut Bus) -> Result {
        let mut tx = bus.add_tx();
        task::spawn(async move {
            loop {
                handle_service_result(
                    &service_id,
                    Duration::from_millis(self.interval_millis),
                    self.loop_(&service_id, &mut tx).await,
                )
                .await;
            }
        });

        Ok(())
    }

    async fn loop_(&self, service_id: &str, tx: &mut Sender) -> Result {
        let now = Utc::now();
        match calc_sunrise_and_set(now, self.secrets.latitude, self.secrets.longitude)? {
            SunriseAndSet::Daylight(sunrise, sunset) => {
                if now < sunrise {
                    Message::new(format!("{}::before::sunrise", service_id))
                        .sensor_title("Time Before Sunrise")
                        .optional_location(self.location.clone())
                        .value(Value::Duration((sunrise - now).num_seconds() as f64))
                        .send_to(tx)
                        .await;
                }
                if now < sunset {
                    Message::new(format!("{}::before::sunset", service_id))
                        .sensor_title("Time Before Sunset")
                        .optional_location(self.location.clone())
                        .value(Value::Duration((sunset - now).num_seconds() as f64))
                        .send_to(tx)
                        .await;
                }
                if sunrise < now {
                    Message::new(format!("{}::after::sunrise", service_id))
                        .sensor_title("Time After Sunrise")
                        .optional_location(self.location.clone())
                        .value(Value::Duration((now - sunrise).num_seconds() as f64))
                        .send_to(tx)
                        .await;
                }
                if sunset < now {
                    Message::new(format!("{}::after::sunset", service_id))
                        .sensor_title("Time After Sunset")
                        .optional_location(self.location.clone())
                        .value(Value::Duration((now - sunset).num_seconds() as f64))
                        .send_to(tx)
                        .await;
                }
            }
            SunriseAndSet::PolarDay => {
                Message::new(format!("{}::polar::day", service_id))
                    .type_(MessageType::ReadNonLogged)
                    .optional_location(self.location.clone())
                    .send_to(tx)
                    .await;
            }
            SunriseAndSet::PolarNight => {
                Message::new(format!("{}::polar::night", service_id))
                    .type_(MessageType::ReadNonLogged)
                    .optional_location(self.location.clone())
                    .send_to(tx)
                    .await;
            }
        }
        Ok(())
    }
}
