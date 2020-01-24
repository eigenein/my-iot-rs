//! Implements generic `Service` trait.
// TODO: move to `core`.

use crate::prelude::*;
use crate::settings::*;
use crate::Result;
use crossbeam_channel::Sender;
use std::sync::{Arc, Mutex};

pub mod automator;
pub mod buienradar;
pub mod clock;
pub mod db;
pub mod nest;
pub mod telegram;

/// Spawn all configured services.
///
/// Returns a vector of all service input message channels.
///
/// - `tx`: output message channel that's used by services to send their messages to.
pub fn spawn_all(
    settings: &Settings,
    db: &Arc<Mutex<Connection>>,
    tx: &Sender<Message>,
) -> Result<Vec<Sender<Message>>> {
    let mut service_txs = Vec::new();

    for (service_id, service_settings) in settings.services.iter() {
        if !settings.disabled_services.contains(service_id.as_str()) {
            info!("Spawning service `{}`…", service_id);
            debug!("Settings `{}`: {:?}", service_id, service_settings);
            let txs = spawn(service_id, service_settings, &db, tx)?;
            debug!("Got {} txs from `{}`", txs.len(), service_id);
            service_txs.extend(txs);
        } else {
            warn!("Service `{}` is disabled.", &service_id);
        }
    }

    Ok(service_txs)
}

/// Spawn a service and return a vector of its message sender sides.
fn spawn(
    service_id: &str,
    settings: &ServiceSettings,
    db: &Arc<Mutex<Connection>>,
    tx: &Sender<Message>,
) -> Result<Vec<Sender<Message>>> {
    // FIXME: I don't really like this large `match`, but I don't know how to fix it properly.
    match settings {
        ServiceSettings::Automator(settings) => automator::spawn(service_id, settings, db, tx),
        ServiceSettings::Buienradar(settings) => buienradar::spawn(service_id, settings, tx),
        ServiceSettings::Clock(settings) => clock::spawn(service_id, settings, tx),
        ServiceSettings::Db(settings) => db::spawn(service_id, settings, db, tx),
        ServiceSettings::Nest(settings) => nest::spawn(service_id, settings, tx),
        ServiceSettings::Telegram(settings) => telegram::spawn(service_id, settings, tx),
    }
}
