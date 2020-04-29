use crate::core::message::Type;
use crate::prelude::*;
use spa::{calc_sunrise_and_set, SunriseAndSet};
use std::thread;
use std::time::Duration;
use uom::si::f64::*;
use uom::si::*;

/// Emits durations to and after sunrise and sunset.
#[derive(Deserialize, Debug, Clone)]
pub struct Solar {
    /// Latitude in [WGS84](https://en.wikipedia.org/wiki/World_Geodetic_System) system, ranging from `-90.0` to `90.0`.
    pub latitude: f64,

    /// Longitude in [WGS84](https://en.wikipedia.org/wiki/World_Geodetic_System) system, ranging from `-180.0` to `180.0`
    pub longitude: f64,

    /// Message interval in milliseconds.
    #[serde(default = "default_interval_ms")]
    pub interval_ms: u64,

    #[serde(default = "default_no_room_title")]
    pub room_title: Option<String>,
}

/// Defaults to one minute.
fn default_interval_ms() -> u64 {
    60000
}

fn default_no_room_title() -> Option<String> {
    None
}

impl Service for Solar {
    fn spawn(&self, service_id: &str, _db: &Arc<Mutex<Connection>>, bus: &mut Bus) -> Result<()> {
        let service_id = service_id.to_string();
        let tx = bus.add_tx();
        let interval = Duration::from_millis(self.interval_ms);
        let settings = self.clone();

        supervisor::spawn(service_id.clone(), tx.clone(), move || -> Result<()> {
            loop {
                let now = Utc::now();
                match calc_sunrise_and_set(now, settings.latitude, settings.longitude)? {
                    SunriseAndSet::Daylight(sunrise, sunset) => {
                        if now < sunrise {
                            Composer::new(format!("{}::before::sunrise", service_id))
                                .type_(Type::ReadSnapshot)
                                .optional_room_title(settings.room_title.clone())
                                .value(Time::new::<time::millisecond>((sunrise - now).num_milliseconds() as f64))
                                .message
                                .send_and_forget(&tx);
                        }
                        if now < sunset {
                            Composer::new(format!("{}::before::sunset", service_id))
                                .type_(Type::ReadSnapshot)
                                .optional_room_title(settings.room_title.clone())
                                .value(Time::new::<time::millisecond>((sunset - now).num_milliseconds() as f64))
                                .message
                                .send_and_forget(&tx);
                        }
                        if sunrise < now {
                            Composer::new(format!("{}::after::sunrise", service_id))
                                .type_(Type::ReadSnapshot)
                                .optional_room_title(settings.room_title.clone())
                                .value(Time::new::<time::millisecond>((now - sunrise).num_milliseconds() as f64))
                                .message
                                .send_and_forget(&tx);
                        }
                        if sunset < now {
                            Composer::new(format!("{}::after::sunset", service_id))
                                .type_(Type::ReadSnapshot)
                                .optional_room_title(settings.room_title.clone())
                                .value(Time::new::<time::millisecond>((now - sunset).num_milliseconds() as f64))
                                .message
                                .send_and_forget(&tx);
                        }
                    }
                    SunriseAndSet::PolarDay => {
                        Composer::new(format!("{}::polar_day", service_id))
                            .type_(Type::ReadNonLogged)
                            .optional_room_title(settings.room_title.clone())
                            .message
                            .send_and_forget(&tx);
                    }
                    SunriseAndSet::PolarNight => {
                        Composer::new(format!("{}::polar_night", service_id))
                            .type_(Type::ReadNonLogged)
                            .optional_room_title(settings.room_title.clone())
                            .message
                            .send_and_forget(&tx);
                    }
                }
                thread::sleep(interval);
            }
        })?;

        Ok(())
    }
}
