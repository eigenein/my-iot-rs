//! Implements generic `Service` trait.

use crate::prelude::*;
use crate::services::*;
use crate::settings::*;
use crate::Result;
use std::sync::{Arc, Mutex};

/// Spawn all configured services.
///
/// Returns a vector of all service input message channels.
///
/// - `tx`: output message channel that's used by services to send their messages to.
pub fn spawn_all(settings: &Settings, db: &Arc<Mutex<Connection>>, bus: &mut Bus) -> Result<()> {
    for (service_id, service_settings) in settings.services.iter() {
        if !settings.disabled_services.contains(service_id.as_str()) {
            info!("Spawning service `{}`…", service_id);
            debug!("Settings `{}`: {:?}", service_id, service_settings);
            spawn(service_id, service_settings, &db, bus)?;
        } else {
            warn!("Service `{}` is disabled.", &service_id);
        }
    }

    Ok(())
}

/// Spawn a service and return a vector of its message sender sides.
fn spawn(service_id: &str, settings: &ServiceSettings, db: &Arc<Mutex<Connection>>, bus: &mut Bus) -> Result<()> {
    // FIXME: I don't really like this large `match`, but I don't know how to fix it properly.
    match settings {
        ServiceSettings::Automator(settings) => automator::spawn(service_id, settings, db, bus),
        ServiceSettings::Buienradar(settings) => buienradar::spawn(service_id, settings, bus),
        ServiceSettings::Clock(settings) => clock::spawn(service_id, settings, bus),
        ServiceSettings::Nest(settings) => nest::spawn(service_id, settings, bus),
        ServiceSettings::Telegram(settings) => telegram::spawn(service_id, settings, bus),
    }
}
