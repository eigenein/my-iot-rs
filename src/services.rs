//! Implements generic `Service` trait.

use crate::db::Db;
use crate::message::*;
use crate::settings::*;
use crate::Result;
use bus::Bus;
use crossbeam_channel::Sender;
use std::sync::{Arc, Mutex};

pub mod automator;
pub mod buienradar;
pub mod clock;
pub mod db;
pub mod nest;
pub mod telegram;

/// A generic service.
pub trait Service: Send {
    /// Spawn service threads.
    ///
    /// Service may spawn as much threads as needed and must take care of their health.
    ///
    /// - `db`: database.
    /// - `tx`: message bus sender.
    /// - `rx`: message bus receiver.
    fn spawn(self: Box<Self>, db: Arc<Mutex<Db>>, tx: &Sender<Message>, rx: &mut Bus<Message>) -> Result<()>;
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
        ServiceSettings::Telegram(settings) => Ok(Box::new(telegram::Telegram::new(service_id, settings)?)),
    }
}
