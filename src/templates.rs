//! Web interface templates.

use crate::core::persistence::{select_actuals, Actual};
use crate::prelude::*;
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct Index<'a> {
    pub crate_version: &'a str,
    pub actuals: Vec<Actual>,
}

#[derive(Template)]
#[template(path = "sensor.html")]
pub struct Sensor<'a> {
    pub crate_version: &'a str,
    pub sensor_id: String,
    pub reading: Reading,
}

impl Index<'_> {
    pub fn new(db: &Connection) -> Result<Self> {
        Ok(Self {
            actuals: select_actuals(db)?,
            crate_version: structopt::clap::crate_version!(),
        })
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
