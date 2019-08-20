//! Implements generic `Service` trait.

use crate::db::Db;
use crate::reading::*;
use crate::settings::*;
use crate::threading::JoinHandle;
use crossbeam_channel::{Receiver, Sender};
use std::sync::{Arc, Mutex};

pub mod buienradar;
pub mod clock;
pub mod db;
pub mod nest;

/// A generic service.
pub trait Service: Send {
    /// Spawn service threads.
    ///
    /// Service may spawn as much threads as needed and must take care of their health.
    ///
    /// - `db`: database.
    /// - `tx`: 0-capacity channel sender.
    /// - `rx`: 0-capacity channel receiver.
    fn spawn(self: Box<Self>, db: Arc<Mutex<Db>>, tx: Sender<Reading>, rx: Receiver<Reading>) -> Vec<JoinHandle>;

    /// Convenience function to send multiple readings at once.
    fn send(&self, tx: &Sender<Reading>, readings: Vec<Reading>) {
        for reading in readings {
            tx.send(reading).unwrap();
        }
    }
}

/// Create a service from the service settings.
pub fn new(service_id: &str, settings: &ServiceSettings) -> Box<dyn Service> {
    // FIXME: the following looks really like a job for dynamic dispatching.
    match settings {
        ServiceSettings::Clock(settings) => Box::new(clock::Clock::new(service_id, settings)),
        ServiceSettings::Db(settings) => Box::new(db::Db::new(service_id, settings)),
        ServiceSettings::Buienradar(settings) => Box::new(buienradar::Buienradar::new(service_id, settings)),
        ServiceSettings::Nest(settings) => Box::new(nest::Nest::new(service_id, settings)),
    }
}
