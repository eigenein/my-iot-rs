use crate::core::persistence::ConnectionExtensions;
use crate::prelude::*;

/// Spawn the persistence thread.
pub fn spawn(db: Arc<Mutex<Connection>>, bus: &mut Bus) -> Result<()> {
    info!("Spawning readings persistence…");
    let tx = bus.add_tx();
    let rx = bus.add_rx();

    crate::core::supervisor::spawn("my-iot::persistence", tx.clone(), move || {
        for message in &rx {
            if let Err(error) = process_message(&message, &db, &tx) {
                error!("{}: {:?}", error, &message);
            }
        }
        unreachable!();
    })?;

    Ok(())
}

/// Process a message.
fn process_message(message: &Message, db: &Arc<Mutex<Connection>>, tx: &Sender<Message>) -> Result<()> {
    info!(
        "{}: {:?} {:?}",
        &message.sensor.sensor_id, &message.type_, &message.reading.value
    );
    debug!("{:?}", &message);
    // TODO: handle `ReadSnapshot`.
    if message.type_ == MessageType::ReadLogged {
        let db = db.lock().unwrap();
        let previous_actual = db.select_last_reading(&message.sensor.sensor_id)?;
        message.upsert_into(&db)?;
        send_messages(&previous_actual, &message, &tx)?;
    }
    Ok(())
}

/// Check if sensor value has been updated or changed and send corresponding messages.
fn send_messages(previous_actual: &Option<Actual>, message: &Message, tx: &Sender<Message>) -> Result<()> {
    if let Some(existing) = previous_actual {
        if message.reading.timestamp > existing.reading.timestamp {
            tx.send(
                Composer::new(format!("{}::update", &message.sensor.sensor_id))
                    .type_(MessageType::ReadNonLogged)
                    .value(message.reading.value.clone())
                    .into(),
            )?;
            if message.reading.value != existing.reading.value {
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
