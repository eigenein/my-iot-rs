//! Event receiver.
use crate::measurement::*;
use log::info;
use std::sync::mpsc::Receiver;

pub fn run(rx: Receiver<Measurement>) {
    for measurement in rx {
        info!("{}: {:?}", &measurement.sensor, &measurement.value);
    }
}
