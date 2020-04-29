use crate::prelude::*;
use crate::supervisor;
use crate::Result;
use chrono::{DateTime, Local};
use crossbeam_channel::Sender;
use eventsource::reqwest::Client;
use rouille::url::Url;
use serde::Deserialize;
use std::collections::HashMap;
use uom::si::f64::*;
use uom::si::*;

const URL: &str = "https://developer-api.nest.com";

#[derive(Deserialize, Debug, Clone)]
pub struct Nest {
    /// Nest API token.
    token: String,
}

impl Service for Nest {
    fn spawn(&self, service_id: &str, _db: &Arc<Mutex<Connection>>, bus: &mut Bus) -> Result<()> {
        let service_id = service_id.to_string();
        let token = self.token.clone();
        let tx = bus.add_tx();

        supervisor::spawn(service_id.clone(), tx.clone(), move || -> Result<()> {
            let client = Client::new(Url::parse_with_params(URL, &[("auth", &token)]).unwrap());
            for event in client {
                if let Ok(event) = event {
                    if let Some(event_type) = event.event_type {
                        if event_type == "put" {
                            send_readings(&service_id, &serde_json::from_str(&event.data)?, &tx)?;
                        }
                    }
                }
            }
            Err(InternalError::new("Event source client is unexpectedly exhausted").into())
        })?;

        Ok(())
    }
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
                .title("Ambient Temperature")
                .room_title(&thermostat.where_name)
                .into(),
        )?;
        tx.send(
            Composer::new(format!("{}::thermostat::{}::humidity", service_id, &id))
                .value(Value::Rh(thermostat.humidity))
                .timestamp(now)
                .title("Humidity")
                .room_title(&thermostat.where_name)
                .into(),
        )?;
    }

    for (id, camera) in event.data.devices.cameras.iter() {
        tx.send(
            Composer::new(format!("{}::camera::{}::snapshot_url", service_id, &id))
                .value(Value::ImageUrl(camera.snapshot_url.clone()))
                .timestamp(now)
                .title("Snapshot")
                .room_title(&camera.where_name)
                .into(),
        )?;

        tx.send(
            Composer::new(format!("{}::camera::{}::is_online", service_id, &id))
                .value(camera.is_online)
                .timestamp(now)
                .title("Camera Online")
                .room_title(&camera.where_name)
                .into(),
        )?;

        if let Some(ref event) = camera.last_event {
            tx.send(
                Composer::new(format!("{}::camera::{}::animated_image_url", service_id, &id))
                    .value(Value::ImageUrl(event.animated_image_url.clone()))
                    .timestamp(event.start_time)
                    .title("Last Event")
                    .room_title(&camera.where_name)
                    .into(),
            )?;
        }
    }

    for (id, alarm) in event.data.devices.smoke_co_alarms.iter() {
        tx.send(
            Composer::new(format!("{}::smoke_co_alarm::{}::is_online", service_id, &id))
                .value(alarm.is_online)
                .timestamp(now)
                .title("Protect Online")
                .room_title(&alarm.where_name)
                .into(),
        )?;
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
    smoke_co_alarms: HashMap<String, NestSmokeCoAlarm>,
}

#[derive(Deserialize, Debug)]
struct NestThermostat {
    ambient_temperature_c: f64,
    humidity: f64,
    name: String,
    where_name: String,
}

#[derive(Deserialize, Debug)]
struct NestCamera {
    snapshot_url: String,
    last_event: Option<NestCameraLastEvent>,
    name: String,
    where_name: String,
    is_online: bool,
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

#[derive(Deserialize, Debug)]
struct NestSmokeCoAlarm {
    where_name: String,
    is_online: bool,
}
