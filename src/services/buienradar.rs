use crate::db::Db;
use crate::measurement::Measurement;
use crate::value::Value;
use serde::Deserialize;
use std::error::Error;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Buienradar JSON feed URL.
const URL: &str = "https://json.buienradar.nl/";
const REFRESH_PERIOD: Duration = Duration::from_millis(60000);

/// Buienradar service settings.
#[derive(Deserialize, Debug)]
pub struct BuienradarSettings {
    /// Station ID. Find a one on https://json.buienradar.nl/.
    station_id: u32,
}

#[derive(Debug)]
pub struct Buienradar {
    station_id: u32,
}

#[derive(Deserialize, Debug)]
pub struct BuienradarFeed {
    actual: BuienradarFeedActual,
}

#[derive(Deserialize, Debug)]
pub struct BuienradarFeedActual {
    #[serde(rename = "stationmeasurements")]
    station_measurements: Vec<BuienradarStationMeasurement>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct BuienradarStationMeasurement {
    #[serde(rename = "stationid")]
    station_id: u32,

    #[serde(rename = "stationname")]
    name: String,
}

impl Buienradar {
    pub fn new(settings: &BuienradarSettings) -> Buienradar {
        Buienradar {
            station_id: settings.station_id,
        }
    }

    /// Fetch measurement for the configured station.
    pub fn fetch(&self) -> Result<BuienradarStationMeasurement, Box<Error>> {
        let body = reqwest::get(URL)?.text()?;
        let feed: BuienradarFeed = serde_json::from_str(&body)?;
        let measurement = feed
            .actual
            .station_measurements
            .iter()
            .find(|measurement| measurement.station_id == self.station_id)
            .ok_or(format!("station {} is not found", self.station_id))?
            .clone();
        Ok(measurement)
    }
}

impl crate::services::Service for Buienradar {
    fn run(&mut self, _db: Arc<Mutex<Db>>, tx: Sender<Measurement>) {
        loop {
            match self.fetch() {
                Ok(measurement) => {
                    #[rustfmt::skip]
                    tx.send(Measurement::new(
                        format!("buienradar:{}:name", self.station_id),
                        Value::Text(measurement.name.clone()),
                        None, // TODO: should use `timestamp`.
                    )).unwrap();
                }
                Err(error) => {
                    log::error!("Buienradar has failed: {}", error);
                }
            }
            thread::sleep(REFRESH_PERIOD);
        }
    }
}
