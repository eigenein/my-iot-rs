use crate::prelude::*;

/// Spawn the persistence thread.
pub fn spawn(db: Arc<Mutex<Connection>>, bus: &mut Bus) -> Result<()> {
    info!("Spawning readings persistence…");
    let rx = bus.add_rx();

    thread::Builder::new()
        .name("system::persistence".into())
        .spawn(move || {
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
        &message.sensor.id, &message.type_, &message.reading.value
    );
    debug!("{:?}", &message);
    // TODO: handle `ReadSnapshot` properly.
    if message.type_ == MessageType::ReadLogged || message.type_ == MessageType::ReadSnapshot {
        message.upsert_into(&db.lock().unwrap())?;
    }
    Ok(())
}
