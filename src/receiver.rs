//! Readings receiver that actually processes all readings coming from services.

use crate::db::*;
use crate::message::*;
use crate::threading;
use crate::value::Value;
use crate::Result;
use bus::Bus;
use chrono::Local;
use crossbeam_channel::Sender;
use log::{debug, info};
use std::sync::{Arc, Mutex};

/// Start readings receiver thread.
pub fn spawn(bus: &mut Bus<Message>, db: Arc<Mutex<Db>>, tx: &Sender<Message>) -> Result<()> {
    info!("Spawning message receiver…");
    let rx = bus.add_rx();
    let tx = tx.clone();

    threading::spawn("my-iot::receiver", move || {
        for message in rx {
            process_message(message, &db, &tx).unwrap();
        }
        unreachable!();
    })?;
    Ok(())
}

/// Process broadcasted message.
fn process_message(message: Message, db: &Arc<Mutex<Db>>, tx: &Sender<Message>) -> Result<()> {
    info!(
        "{}: {:?} {:?}",
        &message.reading.sensor, &message.type_, &message.reading.value,
    );
    debug!("{:?}", &message);
    if message.type_ == Type::Actual {
        let db = db.lock().unwrap();
        check_for_change(&db.select_last_reading(&message.reading.sensor)?, &message, &tx)?;
        db.insert_reading(&message.reading)?;
    }
    Ok(())
}

/// Check if sensor value has changed and send a change message.
fn check_for_change(existing: &Option<Reading>, message: &Message, tx: &Sender<Message>) -> Result<()> {
    if let Some(existing) = existing {
        if existing.timestamp < message.reading.timestamp {
            tx.send(Message {
                type_: Type::OneOff,
                reading: Reading {
                    sensor: format!("{}::update", &message.reading.sensor),
                    value: Value::Update(
                        Box::new(existing.value.clone()),
                        Box::new(message.reading.value.clone()),
                    ),
                    timestamp: Local::now(),
                },
            })?;
        }
    }
    Ok(())
}
