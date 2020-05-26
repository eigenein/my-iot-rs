//! Implements generic `Service` trait.

use crate::prelude::*;
use crate::settings::Settings;

pub trait Service {
    fn spawn(&self, service_id: &str, bus: &mut Bus, db: &Arc<Mutex<Connection>>) -> Result<()>;
}

/// Spawn all configured services.
pub fn spawn_all(settings: &Settings, db: &Arc<Mutex<Connection>>, bus: &mut Bus) -> Result<()> {
    for (service_id, service_settings) in settings.services.iter() {
        info!("Spawning service `{}`…", service_id);
        debug!("Settings `{}`: {:?}", service_id, service_settings);
        crate::services::new(service_settings).spawn(service_id, bus, db)?;
    }
    Ok(())
}
