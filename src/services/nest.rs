use crate::message::{Message, Reading, Type};
use crate::supervisor;
use crate::value::Value;
use crate::Result;
use chrono::{DateTime, Local};
use crossbeam_channel::Sender;
use eventsource::reqwest::Client;
use failure::format_err;
use rouille::url::Url;
use serde::Deserialize;
use std::collections::HashMap;

const URL: &str = "https://developer-api.nest.com";

#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    /// Nest API token.
    token: String,
}

pub fn spawn(service_id: &str, settings: &Settings, tx: &Sender<Message>) -> Result<Vec<Sender<Message>>> {
    let service_id = service_id.to_string();
    let settings = settings.clone();
    let tx = tx.clone();

    supervisor::spawn(format!("my-iot::nest::{}", &service_id), move || -> Result<()> {
        let client = Client::new(Url::parse_with_params(URL, &[("auth", &settings.token)]).unwrap());
        for event in client {
            if let Ok(event) = event {
                if let Some(event_type) = event.event_type {
                    if event_type == "put" {
                        send_readings(&service_id, &serde_json::from_str(&event.data)?, &tx)?;
                    }
                }
            }
        }
        Err(format_err!("Event source client is unexpectedly exhausted"))
    })?;

    Ok(vec![])
}

fn send_readings(service_id: &str, event: &NestEvent, tx: &Sender<Message>) -> Result<()> {
    let now = Local::now();

    for (id, thermostat) in event.data.devices.thermostats.iter() {
        tx.send(Message {
            type_: Type::Actual,
            reading: Reading {
                sensor: format!("{}::thermostat::{}::ambient_temperature", service_id, &id),
                value: Value::Celsius(thermostat.ambient_temperature_c),
                timestamp: now,
            },
        })?;
        tx.send(Message {
            type_: Type::Actual,
            reading: Reading {
                sensor: format!("{}::thermostat::{}::humidity", service_id, &id),
                value: Value::Rh(thermostat.humidity),
                timestamp: now,
            },
        })?;
    }

    for (id, camera) in event.data.devices.cameras.iter() {
        tx.send(Message {
            type_: Type::Actual,
            reading: Reading {
                sensor: format!("{}::camera::{}::snapshot_url", service_id, &id),
                value: Value::ImageUrl(camera.snapshot_url.clone()),
                timestamp: now,
            },
        })?;

        if let Some(ref event) = camera.last_event {
            tx.send(Message::new(
                Type::Actual,
                format!("{}::camera::{}::animated_image_url", service_id, &id),
                Value::ImageUrl(event.animated_image_url.clone()),
                event.start_time,
            ))?;
        }
    }

    Ok(())
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
    last_event: Option<NestCameraLastEvent>,
}

#[derive(Deserialize, Debug)]
struct NestCameraLastEvent {
    has_sound: bool,
    has_motion: bool,
    has_person: bool,
    start_time: DateTime<Local>,
    urls_expire_time: DateTime<Local>,
    animated_image_url: String,
}
