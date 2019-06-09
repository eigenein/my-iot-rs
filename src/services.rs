//! Describes a service trait.
use crate::measurement::*;
use crate::settings::*;
use std::sync::mpsc::Sender;

pub mod clock;

/// A service.
pub trait Service {
    fn run(&mut self, tx: Sender<Measurement>);
}

pub fn new(settings: ServiceSettings) -> Box<dyn Service> {
    Box::new(match settings {
        ServiceSettings::Clock(settings) => clock::Clock::new(&settings),
    })
}
