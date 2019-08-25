use crate::db::Db;
use crate::reading::Reading;
use crate::services::Service;
use crate::threading;
use crate::value::Value;
use crate::Result;
use chrono::Local;
use eventsource::reqwest::Client;
use multiqueue::{BroadcastReceiver, BroadcastSender};
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
    fn spawn(
        self: Box<Self>,
        _db: Arc<Mutex<Db>>,
        tx: &BroadcastSender<Reading>,
        _rx: &BroadcastReceiver<Reading>,
    ) -> Result<()> {
        let tx = tx.clone();
        threading::spawn(self.service_id.clone(), move || loop {
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
    fn send_readings(&self, event: &NestEvent, tx: &BroadcastSender<Reading>) -> Result<()> {
        let now = Local::now();
        for (id, thermostat) in event.data.devices.thermostats.iter() {
            self.send(
                tx,
                vec![
                    Reading {
                        sensor: format!("{}::{}::ambient_temperature", &self.service_id, &id),
                        value: Value::Celsius(thermostat.ambient_temperature_c),
                        timestamp: now,
                        is_persisted: true,
                    },
                    Reading {
                        sensor: format!("{}::{}::humidity", &self.service_id, &id),
                        value: Value::Rh(thermostat.humidity),
                        timestamp: now,
                        is_persisted: true,
                    },
                ],
            )?;
        }
        Ok(())
    }
}

/// Server-side `put` event.
#[derive(Deserialize, Debug)]
pub struct NestEvent {
    data: NestData,
}

#[derive(Deserialize, Debug)]
pub struct NestData {
    devices: NestDevices,
    // TODO: structures.
}

#[derive(Deserialize, Debug)]
pub struct NestDevices {
    thermostats: HashMap<String, NestThermostat>,
    // TODO: smoke_co_alarms
    // TODO: cameras
}

#[derive(Deserialize, Debug)]
pub struct NestThermostat {
    ambient_temperature_c: f64,
    humidity: f64,
}
