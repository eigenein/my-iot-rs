use crate::prelude::*;
use crate::supervisor;
use crate::Result;
use chrono::{DateTime, Local};
use crossbeam_channel::Sender;
use eventsource::reqwest::Client;
use failure::format_err;
use rouille::url::Url;
use serde::Deserialize;
use std::collections::HashMap;
use uom::si::f64::*;
use uom::si::*;

const URL: &str = "https://developer-api.nest.com";

#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    /// Nest API token.
    token: String,
}

pub fn spawn(service_id: &str, settings: &Settings, bus: &mut Bus) -> Result<()> {
    let service_id = service_id.to_string();
    let settings = settings.clone();
    let tx = bus.add_tx();

    supervisor::spawn(
        format!("my-iot::nest::{}", &service_id),
        tx.clone(),
        move || -> Result<()> {
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
        },
    )?;

    Ok(())
}

fn send_readings(service_id: &str, event: &NestEvent, tx: &Sender<Message>) -> Result<()> {
    let now = Local::now();

    for (id, thermostat) in event.data.devices.thermostats.iter() {
        tx.send(
            Composer::new(format!("{}::thermostat::{}::ambient_temperature", service_id, &id))
                .value(
                    ThermodynamicTemperature::new::<thermodynamic_temperature::degree_celsius>(
                        thermostat.ambient_temperature_c,
                    ),
                )
                .timestamp(now)
                .title(format!("{} Ambient Temperature", &thermostat.name))
                .into(),
        )?;
        tx.send(
            Composer::new(format!("{}::thermostat::{}::humidity", service_id, &id))
                .value(Value::Rh(thermostat.humidity))
                .timestamp(now)
                .title(format!("{} Humidity", &thermostat.name))
                .into(),
        )?;
    }

    for (id, camera) in event.data.devices.cameras.iter() {
        tx.send(
            Composer::new(format!("{}::camera::{}::snapshot_url", service_id, &id))
                .value(Value::ImageUrl(camera.snapshot_url.clone()))
                .timestamp(now)
                .title(format!("{} Snapshot", &camera.name))
                .into(),
        )?;

        if let Some(ref event) = camera.last_event {
            tx.send(
                Composer::new(format!("{}::camera::{}::animated_image_url", service_id, &id))
                    .value(Value::ImageUrl(event.animated_image_url.clone()))
                    .timestamp(event.start_time)
                    .title(format!("{} Last Event", &camera.name))
                    .into(),
            )?;
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
    name: String,
}

#[derive(Deserialize, Debug)]
struct NestCamera {
    snapshot_url: String,
    last_event: Option<NestCameraLastEvent>,
    name: String,
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
