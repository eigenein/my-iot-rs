//! Readings receiver that actually processes all readings coming from services.

use crate::db::*;
use crate::reading::*;
use crate::threading::spawn;
use crate::Result;
use log::info;
use multiqueue::BroadcastReceiver;
use std::sync::{Arc, Mutex};

/// Start readings receiver thread.
pub fn start(rx: &BroadcastReceiver<Reading>, db: Arc<Mutex<Db>>) -> Result<()> {
    let rx = rx.add_stream().into_single().unwrap();
    spawn("receiver", move || {
        for reading in rx {
            info!("{}: {:?}", &reading.sensor, &reading.value);
            if reading.is_persisted {
                db.lock().unwrap().insert_reading(&reading).unwrap();
            }
        }
        unreachable!();
    })?;
    Ok(())
}
