//! Readings receiver that actually processes all readings coming from services.

use crate::db::*;
use crate::reading::*;
use log::info;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

/// Run the receiver.
pub fn run(rx: Receiver<Reading>, db: Arc<Mutex<Db>>) {
    for reading in rx {
        info!("{}: {:?}", &reading.sensor, &reading.value);
        db.lock().unwrap().insert_reading(&reading);
    }
}
