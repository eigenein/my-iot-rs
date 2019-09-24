//! Readings receiver that actually processes all readings coming from services.

use crate::db::*;
use crate::message::*;
use crate::threading;
use crate::value::Value;
use crate::Result;
use bus::Bus;
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
        let previous_reading = db.select_last_reading(&message.reading.sensor)?;
        db.insert_reading(&message.reading)?;
        send_messages(&previous_reading, &message, &tx)?;
    }
    Ok(())
}

/// Check if sensor value has been updated or changed and send corresponding messages.
fn send_messages(previous_reading: &Option<Reading>, message: &Message, tx: &Sender<Message>) -> Result<()> {
    if let Some(existing) = previous_reading {
        if message.reading.timestamp > existing.timestamp {
            tx.send(Message::now(
                Type::OneOff,
                format!("{}::update", &message.reading.sensor),
                Value::Update(
                    Box::new(existing.value.clone()),
                    Box::new(message.reading.value.clone()),
                ),
            ))?;
            if message.reading.value != existing.value {
                tx.send(Message::now(
                    Type::OneOff,
                    format!("{}::change", &message.reading.sensor),
                    message.reading.value.clone(),
                ))?;
            }
        }
    }
    Ok(())
}
