//! Web interface templates.

use crate::prelude::*;
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct Index<'a> {
    pub crate_version: &'a str,
    pub actuals: Vec<(crate::prelude::Sensor, Reading)>,
}

#[derive(Template)]
#[template(path = "sensor.html")]
pub struct Sensor<'a> {
    pub crate_version: &'a str,
    pub sensor_id: String,
    pub reading: Reading,
}

impl Index<'_> {
    pub fn new(actuals: Vec<(crate::prelude::Sensor, Reading)>) -> Self {
        Self {
            actuals,
            crate_version: structopt::clap::crate_version!(),
        }
    }
}

impl Sensor<'_> {
    pub fn new<S: Into<String>>(sensor_id: S, reading: Reading) -> Self {
        Self {
            reading,
            sensor_id: sensor_id.into(),
            crate_version: structopt::clap::crate_version!(),
        }
    }
}
