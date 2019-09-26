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

/// Spawn a service.
pub fn spawn(
    service_id: &str,
    settings: &ServiceSettings,
    db: &Arc<Mutex<Db>>,
    tx: &Sender<Message>,
    bus: &mut Bus<Message>,
) -> Result<()> {
    // FIXME: I don't really like this large `match` but I don't know how to fix it properly.
    match settings {
        ServiceSettings::Automator(settings) => automator::spawn(service_id, settings, db, tx, bus)?,
        ServiceSettings::Buienradar(settings) => buienradar::spawn(service_id, settings, tx)?,
        ServiceSettings::Clock(settings) => clock::spawn(service_id, settings, tx)?,
        ServiceSettings::Db(settings) => db::spawn(service_id, settings, db, tx)?,
        ServiceSettings::Nest(settings) => nest::spawn(service_id, settings, tx)?,
        ServiceSettings::Telegram(settings) => telegram::spawn(service_id, settings, tx, bus)?,
    }
    Ok(())
}
