//! Web interface templates.

use crate::core::persistence::{select_actuals, Actual};
use crate::prelude::*;
use askama::Template;
use itertools::Itertools;

#[derive(Template)]
#[template(path = "index.html")]
pub struct Index<'a> {
    pub crate_version: &'a str,
    pub actuals: Vec<(Option<String>, Vec<Actual>)>,
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
            actuals: select_actuals(db)?
                .into_iter()
                .group_by(|actual| actual.sensor.room_title.clone())
                .into_iter()
                .map(|(room_title, group)| (room_title, group.collect_vec()))
                .collect_vec(),
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
