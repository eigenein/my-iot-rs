//! Readings receiver that actually processes all readings coming from services.

use crate::db::*;
use crate::message::*;
use crate::threading;
use crate::Result;
use bus::Bus;
use log::{debug, info};
use std::sync::{Arc, Mutex};

/// Start readings receiver thread.
pub fn spawn(bus: &mut Bus<Message>, db: Arc<Mutex<Db>>) -> Result<()> {
    info!("Spawning message receiver…");
    let rx = bus.add_rx();
    threading::spawn("my-iot::receiver", move || {
        for message in rx {
            info!(
                "{}: {:?} {:?}",
                &message.reading.sensor, &message.type_, &message.reading.value
            );
            debug!("{:?}", &message);
            if message.type_ == Type::Actual {
                db.lock().unwrap().insert_reading(&message.reading).unwrap();
            }
        }
        unreachable!();
    })?;
    Ok(())
}
