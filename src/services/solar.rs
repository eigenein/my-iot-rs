use crate::core::message::Type;
use crate::prelude::*;
use spa::{calc_sunrise_and_set, SunriseAndSet};
use std::thread;
use std::time::Duration;
use uom::si::f64::*;
use uom::si::*;

/// Emits durations to and after sunrise and sunset.
#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Solar {
    /// Message interval in milliseconds.
    #[serde(default = "default_interval_ms")]
    pub interval_ms: u64,

    #[serde(default)]
    pub room_title: Option<String>,

    pub secrets: Secrets,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Secrets {
    /// Latitude in [WGS84](https://en.wikipedia.org/wiki/World_Geodetic_System) system, ranging from `-90.0` to `90.0`.
    pub latitude: f64,

    /// Longitude in [WGS84](https://en.wikipedia.org/wiki/World_Geodetic_System) system, ranging from `-180.0` to `180.0`
    pub longitude: f64,
}

/// Defaults to one minute.
fn default_interval_ms() -> u64 {
    60000
}

impl Solar {
    pub fn spawn(self, service_id: String, bus: &mut Bus) -> Result<()> {
        let tx = bus.add_tx();
        let interval = Duration::from_millis(self.interval_ms);
        let ttl = chrono::Duration::days(1);

        thread::Builder::new().name(service_id.clone()).spawn(move || loop {
            let now = Utc::now();
            match calc_sunrise_and_set(now, self.secrets.latitude, self.secrets.longitude) {
                Ok(SunriseAndSet::Daylight(sunrise, sunset)) => {
                    if now < sunrise {
                        Message::new(format!("{}::before::sunrise", service_id))
                            .type_(Type::ReadSnapshot)
                            .sensor_title("Time Before Sunrise")
                            .optional_room_title(self.room_title.clone())
                            .value(Time::new::<time::millisecond>((sunrise - now).num_milliseconds() as f64))
                            .expires_in(ttl)
                            .send_and_forget(&tx);
                    }
                    if now < sunset {
                        Message::new(format!("{}::before::sunset", service_id))
                            .type_(Type::ReadSnapshot)
                            .sensor_title("Time Before Sunset")
                            .optional_room_title(self.room_title.clone())
                            .value(Time::new::<time::millisecond>((sunset - now).num_milliseconds() as f64))
                            .expires_in(ttl)
                            .send_and_forget(&tx);
                    }
                    if sunrise < now {
                        Message::new(format!("{}::after::sunrise", service_id))
                            .type_(Type::ReadSnapshot)
                            .sensor_title("Time After Sunrise")
                            .optional_room_title(self.room_title.clone())
                            .value(Time::new::<time::millisecond>((now - sunrise).num_milliseconds() as f64))
                            .expires_in(ttl)
                            .send_and_forget(&tx);
                    }
                    if sunset < now {
                        Message::new(format!("{}::after::sunset", service_id))
                            .type_(Type::ReadSnapshot)
                            .sensor_title("Time After Sunset")
                            .optional_room_title(self.room_title.clone())
                            .value(Time::new::<time::millisecond>((now - sunset).num_milliseconds() as f64))
                            .expires_in(ttl)
                            .send_and_forget(&tx);
                    }
                }
                Ok(SunriseAndSet::PolarDay) => {
                    Message::new(format!("{}::polar_day", service_id))
                        .type_(Type::ReadNonLogged)
                        .optional_room_title(self.room_title.clone())
                        .expires_in(ttl)
                        .send_and_forget(&tx);
                }
                Ok(SunriseAndSet::PolarNight) => {
                    Message::new(format!("{}::polar_night", service_id))
                        .type_(Type::ReadNonLogged)
                        .optional_room_title(self.room_title.clone())
                        .expires_in(ttl)
                        .send_and_forget(&tx);
                }
                Err(error) => error!("Failed to calculate sunrise and sunset: {}", error.to_string()),
            }
            thread::sleep(interval);
        })?;

        Ok(())
    }
}
