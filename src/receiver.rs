//! Readings receiver that actually processes all readings coming from services.

use crate::db::*;
use crate::reading::*;
use crate::threading::spawn;
use crate::Result;
use log::info;
use multiqueue2::BroadcastReceiver;
use std::sync::{Arc, Mutex};

/// Start readings receiver thread.
pub fn start(rx: &BroadcastReceiver<Message>, db: Arc<Mutex<Db>>) -> Result<()> {
    let rx = rx.add_stream().into_single().unwrap();
    spawn("receiver", move || {
        for message in rx {
            info!("{}: {:?}", &message.reading.sensor, &message.reading.value);
            if message.type_ == Type::Actual {
                db.lock().unwrap().insert_reading(&message.reading).unwrap();
            }
        }
        unreachable!();
    })?;
    Ok(())
}
