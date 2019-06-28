//! Measurement receiver.
use crate::db::*;
use crate::measurement::*;
use log::info;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

/// Run the receiver.
pub fn run(rx: Receiver<Measurement>, db: Arc<Mutex<Db>>) {
    for measurement in rx {
        info!("{}: {:?}", &measurement.sensor, &measurement.value);
        db.lock().unwrap().insert_measurement(&measurement);
    }
}
