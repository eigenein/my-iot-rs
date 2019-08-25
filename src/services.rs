//! Implements generic `Service` trait.

use crate::db::Db;
use crate::reading::*;
use crate::settings::*;
use crate::Result;
use crossbeam_channel::{Receiver, Sender};
use std::sync::{Arc, Mutex};

pub mod automator;
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
    fn spawn(self: Box<Self>, db: Arc<Mutex<Db>>, tx: Sender<Reading>, rx: Receiver<Reading>) -> Result<()>;

    /// Convenience function to send multiple readings at once.
    fn send(&self, tx: &Sender<Reading>, readings: Vec<Reading>) -> Result<()> {
        for reading in readings {
            tx.send(reading)?;
        }
        Ok(())
    }
}

/// Create a service from the service settings.
pub fn new(service_id: &str, settings: &ServiceSettings) -> Result<Box<dyn Service>> {
    // FIXME: the following looks really like a job for dynamic dispatching.
    match settings {
        ServiceSettings::Clock(settings) => Ok(Box::new(clock::Clock::new(service_id, settings))),
        ServiceSettings::Db(settings) => Ok(Box::new(db::Db::new(service_id, settings))),
        ServiceSettings::Buienradar(settings) => Ok(Box::new(buienradar::Buienradar::new(service_id, settings)?)),
        ServiceSettings::Nest(settings) => Ok(Box::new(nest::Nest::new(service_id, settings))),
        ServiceSettings::Automator(settings) => Ok(Box::new(automator::Automator::new(service_id, settings))),
    }
}
