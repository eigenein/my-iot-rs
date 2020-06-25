//! Implements generic `Service` trait.

use crate::prelude::*;
use crate::settings::{Service, Settings};

/// Spawn all the configured services.
pub fn spawn_all(settings: &Settings, service_ids: &Option<Vec<String>>, bus: &mut Bus) -> Result<()> {
    for (service_id, service) in settings.services.iter() {
        if let Some(service_ids) = service_ids {
            if !service_ids.contains(service_id) {
                warn!("`{}` is not included in the `--service-id` option", service_id);
                continue;
            }
        }

        info!("Spawning service `{}`…", service_id);
        debug!("Settings `{}`: {:?}", service_id, service);
        let service_id = service_id.clone();
        match service.clone() {
            Service::Buienradar(buienradar) => buienradar.spawn(service_id, bus),
            Service::Clock(clock) => clock.spawn(service_id, bus),
            Service::Lua(lua) => lua.spawn(service_id, bus, settings.services.clone()),
            Service::Solar(solar) => solar.spawn(service_id, bus),
            Service::Tado(tado) => tado.spawn(service_id, bus),
            Service::Telegram(telegram) => telegram.spawn(service_id, bus),
            Service::YouLess(youless) => youless.spawn(service_id, bus),
        }?;
    }
    Ok(())
}
