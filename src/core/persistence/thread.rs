use crate::prelude::*;

/// Spawn the persistence thread.
pub fn spawn(db: Arc<Mutex<Connection>>, bus: &mut Bus) -> Result<()> {
    info!("Spawning readings persistence…");
    let rx = bus.add_rx();

    crate::core::supervisor::spawn("persistence", bus.add_tx(), move || {
        for message in &rx {
            if let Err(error) = process_message(&message, &db) {
                error!("{}: {:?}", error, &message);
            }
        }
        unreachable!();
    })?;

    Ok(())
}

/// Process a message.
fn process_message(message: &Message, db: &Arc<Mutex<Connection>>) -> Result<()> {
    info!(
        "{}: {:?} {:?}",
        &message.sensor.sensor_id, &message.type_, &message.reading.value
    );
    debug!("{:?}", &message);
    // TODO: handle `ReadSnapshot` properly.
    if message.type_ == MessageType::ReadLogged || message.type_ == MessageType::ReadSnapshot {
        let db = db.lock().unwrap();
        message.upsert_into(&db)?;
    }
    Ok(())
}
