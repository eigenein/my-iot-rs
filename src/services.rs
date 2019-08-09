//! Implements generic `Service` trait.

use crate::db::Db;
use crate::measurement::*;
use crate::settings::*;
use std::fmt::Debug;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

pub mod buienradar;
pub mod clock;
pub mod db;

/// A generic service.
pub trait Service: Debug + Send {
    fn run(&mut self, db: Arc<Mutex<Db>>, tx: Sender<Measurement>) -> !;

    /// Convenience function to send multiple measurements at once.
    fn send(&self, tx: &Sender<Measurement>, measurements: Vec<Measurement>) {
        for measurement in measurements {
            tx.send(measurement).unwrap();
        }
    }
}

/// Create a service from the service settings.
pub fn new(settings: &ServiceSettings) -> Box<dyn Service> {
    match settings {
        ServiceSettings::Clock(settings) => Box::new(clock::Clock::new(settings)),
        ServiceSettings::Db(settings) => Box::new(db::Db::new(settings)),
        ServiceSettings::Buienradar(settings) => Box::new(buienradar::Buienradar::new(settings)),
    }
}
