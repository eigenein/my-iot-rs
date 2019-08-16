//! Readings receiver that actually processes all readings coming from services.

use crate::db::*;
use crate::reading::*;
use crate::threading::ArcMutex;
use crossbeam_channel::Receiver;
use log::{error, info};
use std::sync::{Arc, Mutex};
use std::thread;

/// Start readings receiver thread.
pub fn start(rx: Receiver<Reading>, db: ArcMutex<Db>) {
    thread::spawn(move || run(rx, db));
}

/// Run the receiver.
fn run(rx: Receiver<Reading>, db: Arc<Mutex<Db>>) {
    for reading in rx.iter() {
        info!("{}: {:?}", &reading.sensor, &reading.value);
        if reading.is_persisted {
            db.lock().unwrap().insert_reading(&reading);
        }
    }
    error!("Reading receiver has stopped.");
}
