use crate::prelude::*;
use crate::settings::ServiceSettings;

pub mod buienradar;
pub mod clock;
pub mod db;
pub mod lua;
pub mod nest;
pub mod solar;
pub mod telegram;

pub fn new(settings: &ServiceSettings) -> Box<&dyn Service> {
    Box::new(match settings {
        ServiceSettings::Buienradar(service) => service,
        ServiceSettings::Clock(service) => service,
        ServiceSettings::Lua(service) => service,
        ServiceSettings::Nest(service) => service,
        ServiceSettings::Telegram(service) => service,
        ServiceSettings::Solar(service) => service,
    })
}
