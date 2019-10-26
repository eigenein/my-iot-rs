//! Readings persistence.

use crate::db::*;
use crate::message::*;
use crate::supervisor;
use crate::Result;
use crossbeam_channel::Sender;
use log::{debug, info};
use std::sync::{Arc, Mutex};

/// Start the persistence thread.
pub fn spawn(db: Arc<Mutex<Db>>, tx: &Sender<Message>) -> Result<Sender<Message>> {
    info!("Spawning readings persistence…");
    let tx = tx.clone();
    let (out_tx, rx) = crossbeam_channel::unbounded::<Message>();

    supervisor::spawn("my-iot::persistence", tx.clone(), move || {
        for message in &rx {
            process_message(message, &db, &tx).unwrap();
        }
        unreachable!();
    })?;

    Ok(out_tx)
}

/// Process a message.
fn process_message(message: Message, db: &Arc<Mutex<Db>>, tx: &Sender<Message>) -> Result<()> {
    info!("{}: {:?} {:?}", &message.sensor, &message.type_, &message.value,);
    debug!("{:?}", &message);
    if message.type_ == Type::ReadLogged {
        let db = db.lock().unwrap();
        let previous_reading = db.select_last_reading(&message.sensor)?;
        db.insert_reading(&message)?;
        send_messages(&previous_reading, &message, &tx)?;
    }
    Ok(())
}

/// Check if sensor value has been updated or changed and send corresponding messages.
fn send_messages(previous_reading: &Option<Message>, message: &Message, tx: &Sender<Message>) -> Result<()> {
    if let Some(existing) = previous_reading {
        if message.timestamp > existing.timestamp {
            tx.send(
                Composer::new(format!("{}::update", &message.sensor))
                    .type_(Type::ReadNonLogged)
                    .value(message.value.clone())
                    .into(),
            )?;
            if message.value != existing.value {
                tx.send(
                    Composer::new(format!("{}::change", &message.sensor))
                        .type_(Type::ReadNonLogged)
                        .value(message.value.clone())
                        .into(),
                )?;
            }
        }
    }
    Ok(())
}
