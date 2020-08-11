use crate::prelude::*;
use crate::settings::{Service, Settings};

pub mod anomaly;
pub mod buienradar;
pub mod clock;
pub mod db;
pub mod helpers;
pub mod openweather;
pub mod prelude;
pub mod rhai;
pub mod ring;
pub mod solar;
pub mod tado;
pub mod telegram;
pub mod threshold;
pub mod youless;

/// Spawn all the configured services.
pub async fn spawn_all(
    settings: &Settings,
    service_ids: &Option<Vec<String>>,
    bus: &mut Bus,
    db: &Connection,
) -> Result {
    for (service_id, service) in settings.services.iter() {
        if let Some(service_ids) = service_ids {
            if !service_ids.contains(service_id) {
                warn!("`{}` is not included in the `--service-id` option", service_id);
                continue;
            }
        }

        info!("Spawning service `{}`…", service_id);
        debug!("Settings `{}`: {:?}", service_id, service);
        if let Err(error) = {
            let service_id = service_id.clone();
            match service.clone() {
                Service::Buienradar(service) => service.spawn(service_id, bus),
                Service::Clock(service) => service.spawn(service_id, bus).await,
                Service::OpenWeather(service) => service.spawn(service_id, bus),
                Service::Rhai(service) => service.spawn(service_id, bus, settings.services.clone()),
                Service::Ring(service) => service.spawn(service_id, db.clone(), bus),
                Service::SimpleAnomalyDetector(service) => service.spawn(service_id, bus, db).await,
                Service::Solar(service) => service.spawn(service_id, bus),
                Service::Tado(service) => service.spawn(service_id, bus).await,
                Service::Telegram(service) => service.spawn(service_id, bus),
                Service::Threshold(service) => service.spawn(service_id, bus),
                Service::YouLess(service) => service.spawn(service_id, bus),
            }
        } {
            error!("Failed to spawn `{}`: {}", service_id, error.to_string());
        }
    }
    Ok(())
}
