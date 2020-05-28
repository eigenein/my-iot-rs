//! Implements generic `Service` trait.

use crate::prelude::*;
use crate::settings::{Service, Settings};

/// Spawn all the configured services.
pub fn spawn_all<'env>(scope: &Scope<'env>, settings: &'env Settings, bus: &mut Bus) -> Result<()> {
    for (service_id, service) in settings.services.iter() {
        info!("Spawning service `{}`…", service_id);
        debug!("Settings `{}`: {:?}", service_id, service);
        match service {
            Service::Buienradar(buienradar) => buienradar.spawn(scope, service_id, bus),
            Service::Clock(clock) => clock.spawn(scope, service_id, bus),
            Service::Lua(lua) => lua.spawn(scope, service_id, bus, &settings.services),
            Service::Nest(nest) => nest.spawn(scope, service_id, bus),
            Service::Solar(solar) => solar.spawn(scope, service_id, bus),
            Service::Telegram(telegram) => telegram.spawn(scope, service_id, bus),
        }?;
    }
    Ok(())
}
