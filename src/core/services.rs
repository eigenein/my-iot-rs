//! Implements generic `Service` trait.

use crate::prelude::*;
use crate::settings::{Service, Settings};

/// Spawn all the configured services.
pub fn spawn_all(settings: &Settings, _db: &Arc<Mutex<Connection>>, bus: &mut Bus) -> Result<()> {
    for (service_id, service) in settings.services.iter() {
        info!("Spawning service `{}`…", service_id);
        debug!("Settings `{}`: {:?}", service_id, service);
        match service {
            Service::Buienradar(service) => service.spawn(service_id, bus),
            Service::Clock(service) => service.spawn(service_id, bus),
            Service::Lua(service) => service.spawn(service_id, bus, &settings.services),
            Service::Nest(service) => service.spawn(service_id, bus),
            Service::Solar(service) => service.spawn(service_id, bus),
            Service::Telegram(service) => service.spawn(service_id, bus),
        }?;
    }
    Ok(())
}
