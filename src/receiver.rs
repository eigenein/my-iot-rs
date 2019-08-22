//! Readings receiver that actually processes all readings coming from services.

use crate::db::*;
use crate::reading::*;
use crate::threading::{spawn, ArcMutex};
use crate::Result;
use crossbeam_channel::Receiver;
use log::{error, info};
use std::sync::{Arc, Mutex};

/// Start readings receiver thread.
pub fn start(rx: Receiver<Reading>, db: ArcMutex<Db>) -> Result<()> {
    spawn("receiver", move || run(rx, db))?;
    Ok(())
}

/// Run the receiver.
fn run(rx: Receiver<Reading>, db: Arc<Mutex<Db>>) {
    for reading in rx.iter() {
        info!("{}: {:?}", &reading.sensor, &reading.value);
        if reading.is_persisted {
            db.lock().unwrap().insert_reading(&reading).unwrap();
        }
    }
    error!("Reading receiver has stopped.");
}
