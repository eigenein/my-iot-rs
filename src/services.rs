use crate::prelude::*;
use crate::settings::{Service, Settings};
use lazy_static::lazy_static;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Method;
use std::time::Duration;

pub mod buienradar;
pub mod clock;
pub mod db;
pub mod openweather;
pub mod rhai;
pub mod ring;
pub mod solar;
pub mod tado;
pub mod telegram;
pub mod youless;

lazy_static! {
    /// `Client` instance used to make requests to all services.
    static ref CLIENT: Client = build_client().expect("Failed to build a client");
}

/// Spawn all the configured services.
pub fn spawn_all(settings: &Settings, service_ids: &Option<Vec<String>>, bus: &mut Bus, db: &Connection) -> Result {
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
                Service::Buienradar(buienradar) => buienradar.spawn(service_id, bus),
                Service::Clock(clock) => clock.spawn(service_id, bus),
                Service::OpenWeather(openweather) => openweather.spawn(service_id, bus),
                Service::Rhai(rhai) => rhai.spawn(service_id, bus, settings.services.clone()),
                Service::Solar(solar) => solar.spawn(service_id, bus),
                Service::Tado(tado) => tado.spawn(service_id, bus),
                Service::Telegram(telegram) => telegram.spawn(service_id, bus),
                Service::YouLess(youless) => youless.spawn(service_id, bus),
                Service::Ring(ring) => ring.spawn(service_id, bus, db),
            }
        } {
            error!("Failed to spawn `{}`: {}", service_id, error.to_string());
        }
    }
    Ok(())
}

/// Builds an HTTP client to use with a service.
fn build_client() -> Result<Client> {
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_static(USER_AGENT));
    Ok(Client::builder()
        .gzip(true)
        .use_rustls_tls()
        .default_headers(headers)
        .timeout(Duration::from_secs(10))
        .pool_idle_timeout(Some(Duration::from_secs(300)))
        .build()?)
}

/// Deserializes a Unix time into `DateTime<Local>`.
fn deserialize_timestamp<'de, D: Deserializer<'de>>(deserializer: D) -> Result<DateTime<Local>, D::Error> {
    Ok(Local.timestamp(i64::deserialize(deserializer)?, 0))
}

/// Generic function to call a JSON API.
fn call_json_api<U, R>(method: Method, access_token: &str, url: U) -> Result<R>
where
    U: AsRef<str> + std::fmt::Display,
    R: DeserializeOwned,
{
    debug!("Calling {}…", url);
    let response = CLIENT
        .request(method, url.as_ref())
        .header("Authorization", format!("Bearer {}", access_token))
        .send()?
        .error_for_status()?
        .json()?;
    debug!("Finished {}.", url);
    Ok(response)
}
