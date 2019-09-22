use crate::db::Db;
use crate::message::{Message, Reading, Type};
use crate::services::Service;
use crate::threading;
use crate::value::Value;
use crate::Result;
use bus::Bus;
use chrono::Local;
use crossbeam_channel::Sender;
use eventsource::reqwest::Client;
use rouille::url::Url;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

const URL: &str = "https://developer-api.nest.com";

pub struct Nest {
    service_id: String,
    token: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    /// Nest API token.
    token: String,
}

impl Nest {
    pub fn new(service_id: &str, settings: &Settings) -> Nest {
        Nest {
            service_id: service_id.into(),
            token: settings.token.clone(),
        }
    }
}

impl Service for Nest {
    fn spawn(self: Box<Self>, _db: Arc<Mutex<Db>>, tx: &Sender<Message>, _rx: &mut Bus<Message>) -> Result<()> {
        let tx = tx.clone();
        threading::spawn(format!("my-iot::nest:{}", &self.service_id), move || loop {
            let client = Client::new(Url::parse_with_params(URL, &[("auth", &self.token)]).unwrap());
            for event in client {
                if let Ok(event) = event {
                    if let Some(event_type) = event.event_type {
                        if event_type == "put" {
                            self.send_readings(&serde_json::from_str(&event.data).unwrap(), &tx)
                                .unwrap();
                        }
                    }
                }
            }
        })?;
        Ok(())
    }
}

impl Nest {
    fn send_readings(&self, event: &NestEvent, tx: &Sender<Message>) -> Result<()> {
        let now = Local::now();

        for (id, thermostat) in event.data.devices.thermostats.iter() {
            tx.try_send(Message {
                type_: Type::Actual,
                reading: Reading {
                    sensor: format!("{}::thermostat::{}::ambient_temperature", &self.service_id, &id),
                    value: Value::Celsius(thermostat.ambient_temperature_c),
                    timestamp: now,
                },
            })?;
            tx.try_send(Message {
                type_: Type::Actual,
                reading: Reading {
                    sensor: format!("{}::thermostat::{}::humidity", &self.service_id, &id),
                    value: Value::Rh(thermostat.humidity),
                    timestamp: now,
                },
            })?;
        }

        for (id, camera) in event.data.devices.cameras.iter() {
            tx.try_send(Message {
                type_: Type::Actual,
                reading: Reading {
                    sensor: format!("{}::camera::{}::snapshot_url", &self.service_id, &id),
                    value: Value::ImageUrl(camera.snapshot_url.clone()),
                    timestamp: now,
                },
            })?;
        }

        Ok(())
    }
}

/// Server-side `put` event.
#[derive(Deserialize, Debug)]
struct NestEvent {
    data: NestData,
}

#[derive(Deserialize, Debug)]
struct NestData {
    devices: NestDevices,
    // TODO: structures.
}

#[derive(Deserialize, Debug)]
struct NestDevices {
    thermostats: HashMap<String, NestThermostat>,
    cameras: HashMap<String, NestCamera>,
    // TODO: smoke_co_alarms
}

#[derive(Deserialize, Debug)]
struct NestThermostat {
    ambient_temperature_c: f64,
    humidity: f64,
}

#[derive(Deserialize, Debug)]
struct NestCamera {
    snapshot_url: String,
}
