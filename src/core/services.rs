//! Implements generic `Service` trait.

use crate::prelude::*;
use crate::settings::*;
use crate::Result;
use std::sync::{Arc, Mutex};

pub trait Service {
    fn spawn(&self, service_id: &str, db: &Arc<Mutex<Connection>>, bus: &mut Bus) -> Result<()>;
}

/// Spawn all configured services.
///
/// Returns a vector of all service input message channels.
///
/// - `tx`: output message channel that's used by services to send their messages to.
pub fn spawn_all(settings: &Settings, db: &Arc<Mutex<Connection>>, bus: &mut Bus) -> Result<()> {
    for (service_id, service_settings) in settings.services.iter() {
        info!("Spawning service `{}`…", service_id);
        debug!("Settings `{}`: {:?}", service_id, service_settings);
        spawn(service_id, service_settings, &db, bus)?;
    }
    Ok(())
}

/// Spawn a service and return a vector of its message sender sides.
fn spawn(service_id: &str, settings: &ServiceSettings, db: &Arc<Mutex<Connection>>, bus: &mut Bus) -> Result<()> {
    let service: &dyn Service = match settings {
        ServiceSettings::Buienradar(service) => service,
        ServiceSettings::Clock(service) => service,
        ServiceSettings::Lua(service) => service,
        ServiceSettings::Nest(service) => service,
        ServiceSettings::Telegram(service) => service,
    };
    service.spawn(service_id, db, bus)
}
