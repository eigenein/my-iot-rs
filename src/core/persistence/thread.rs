use crate::core::persistence::*;
use crate::prelude::*;

/// Spawn the persistence thread.
pub fn spawn(db: Arc<Mutex<Connection>>, tx: &Sender<Message>) -> Result<Sender<Message>> {
    info!("Spawning readings persistence…");
    let tx = tx.clone();
    let (out_tx, rx) = crossbeam_channel::unbounded::<Message>();

    crate::core::supervisor::spawn("my-iot::persistence", tx.clone(), move || {
        for message in &rx {
            process_message(message, &db, &tx).unwrap();
        }
        unreachable!();
    })?;

    Ok(out_tx)
}

/// Process a message.
fn process_message(message: Message, db: &Arc<Mutex<Connection>>, tx: &Sender<Message>) -> Result<()> {
    info!(
        "{}: {:?} {:?}",
        &message.sensor.sensor_id, &message.type_, &message.reading.value
    );
    debug!("{:?}", &message);
    // TODO: handle `ReadSnapshot`.
    if message.type_ == MessageType::ReadLogged {
        let db = db.lock()?;
        let previous_reading = select_last_reading(&db, &message.sensor.sensor_id)?;
        upsert_reading(&db, &message.sensor, &message.reading)?;
        send_messages(&previous_reading, &message, &tx)?;
    }
    Ok(())
}

/// Check if sensor value has been updated or changed and send corresponding messages.
fn send_messages(previous_reading: &Option<Reading>, message: &Message, tx: &Sender<Message>) -> Result<()> {
    if let Some(existing) = previous_reading {
        if message.reading.timestamp > existing.timestamp {
            tx.send(
                Composer::new(format!("{}::update", &message.sensor.sensor_id))
                    .type_(MessageType::ReadNonLogged)
                    .value(message.reading.value.clone())
                    .into(),
            )?;
            if message.reading.value != existing.value {
                tx.send(
                    Composer::new(format!("{}::change", &message.sensor.sensor_id))
                        .type_(MessageType::ReadNonLogged)
                        .value(message.reading.value.clone())
                        .into(),
                )?;
            }
        }
    }
    Ok(())
}
