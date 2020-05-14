//! Web interface templates.

use crate::prelude::*;
use askama::Template;
use itertools::Itertools;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    pub crate_version: &'a str,
    pub actuals: Vec<(Option<String>, Vec<Actual>)>,
}

#[derive(Template)]
#[template(path = "sensor.html")]
pub struct SensorTemplate<'a> {
    pub crate_version: &'a str,
    pub actual: Actual,
}

impl IndexTemplate<'_> {
    pub fn new(db: &Connection, max_sensor_age_ms: i64) -> Result<Self> {
        Ok(Self {
            actuals: db
                .select_actuals(max_sensor_age_ms)?
                .into_iter()
                .group_by(|actual| actual.sensor.room_title.clone())
                .into_iter()
                .map(|(room_title, group)| (room_title, group.collect_vec()))
                .collect_vec(),
            crate_version: structopt::clap::crate_version!(),
        })
    }
}

impl SensorTemplate<'_> {
    pub fn new(actual: Actual) -> Self {
        Self {
            actual,
            crate_version: structopt::clap::crate_version!(),
        }
    }
}

mod filters {
    use chrono::{DateTime, Local};

    pub fn format_datetime(datetime: &DateTime<Local>) -> askama::Result<String> {
        Ok(datetime.format("%b %d, %H:%M:%S").to_string())
    }
}
