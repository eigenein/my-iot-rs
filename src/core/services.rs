//! Implements generic `Service` trait.

use crate::prelude::*;
use crate::settings::*;
use std::sync::{Arc, Mutex};

pub trait Service {
    fn spawn(&self, service_id: &str, bus: &mut Bus, db: &Arc<Mutex<Connection>>) -> Result<()>;
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
        crate::services::new(service_settings).spawn(service_id, bus, db)?;
    }
    Ok(())
}
