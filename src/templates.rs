//! Web interface templates.

use crate::prelude::*;
use askama::Template;
use itertools::Itertools;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub actuals: Vec<(Option<String>, Vec<Actual>)>,
}

impl IndexTemplate {
    pub fn new(db: &Connection, max_sensor_age_ms: i64) -> Result<Self> {
        Ok(Self {
            actuals: db
                .select_actuals(max_sensor_age_ms)?
                .into_iter()
                .group_by(|actual| actual.sensor.room_title.clone())
                .into_iter()
                .map(|(room_title, group)| (room_title, group.collect_vec()))
                .collect_vec(),
        })
    }
}

#[derive(Template)]
#[template(path = "sensor.html")]
pub struct SensorTemplate {
    pub actual: Actual,
}

impl SensorTemplate {
    pub fn new(actual: Actual) -> Self {
        Self { actual }
    }
}

#[derive(Template)]
#[template(path = "partials/navbar.html")]
pub struct NavbarTemplate<'a> {
    pub selected_item: &'a str,
}

impl<'a> NavbarTemplate<'a> {
    pub fn new(selected_item: &'a str) -> Self {
        NavbarTemplate { selected_item }
    }
}

fn crate_version() -> &'static str {
    structopt::clap::crate_version!()
}

mod filters {
    use chrono::{DateTime, Local};

    pub fn format_datetime(datetime: &DateTime<Local>) -> askama::Result<String> {
        Ok(datetime.format("%b %d, %H:%M:%S").to_string())
    }
}
