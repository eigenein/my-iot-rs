//! Describes a service trait.
use crate::db::Db;
use crate::measurement::*;
use crate::settings::*;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

pub mod clock;
pub mod db;

/// A service.
pub trait Service {
    fn run(&mut self, db: Arc<Mutex<Db>>, tx: Sender<Measurement>);
}

/// Create a service from the settings.
pub fn new(settings: ServiceSettings) -> Box<dyn Service> {
    match settings {
        ServiceSettings::Clock(settings) => Box::new(clock::Clock::new(&settings)),
        ServiceSettings::Db(settings) => Box::new(db::Db::new(&settings)),
    }
}
