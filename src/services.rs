//! Describes a service trait.
use crate::event::Event;
use crate::settings::*;
use std::sync::mpsc::Sender;

pub mod clock;

/// A service.
pub trait Service {
    fn run(&mut self, tx: Sender<Event>);
}

pub fn new(settings: ServiceSettings) -> Box<dyn Service> {
    Box::new(match settings {
        ServiceSettings::Clock(settings) => clock::Clock::new(&settings),
    })
}
