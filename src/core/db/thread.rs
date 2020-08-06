//! Database persistence thread.

use crate::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// TODO: make configurable.
const COMMIT_INTERVAL_MILLIS: u64 = 1000;

/// Spawn the persistence thread.
pub fn spawn(db: Connection, bus: &mut Bus) -> Result {
    info!("Spawning readings persistence…");
    let rx = bus.add_rx();
    let buffer = Arc::new(Mutex::new(Vec::<Message>::new()));

    {
        let buffer = buffer.clone();
        thread::Builder::new()
            .name("system::persistence::receiver".into())
            .spawn(move || {
                for message in &rx {
                    buffer.lock().unwrap().push(message);
                }
                unreachable!();
            })?;
    }

    thread::Builder::new()
        .name("system::persistence::executor".into())
        .spawn(move || loop {
            sleep(Duration::from_millis(COMMIT_INTERVAL_MILLIS));

            let messages = {
                // Acquire a lock, quickly clone the messages (if any) and release the lock.
                let mut buffer = buffer.lock().unwrap();
                if buffer.is_empty() {
                    continue;
                }
                // TODO: is it possible to move out the messages instead of clone+clear?
                let messages = buffer.clone();
                buffer.clear();
                messages
            };

            // Now `messages` is a clone, thus we can perform a slow operation.
            let start_time = Instant::now();
            if let Err(error) = upsert_messages(&db, messages) {
                error!("could not upsert the messages: {}", error);
            }
            info!("Took {:.1?}.", start_time.elapsed());
        })?;

    Ok(())
}

/// Upserts the messages within a single transaction.
///
/// Inserting messages one by one is quite slow on low-performance boards.
/// Thus, I spin up a separate thread which accumulates incoming messages
/// and periodically upserts them all within a single transaction.
fn upsert_messages(db: &Connection, messages: Vec<Message>) -> Result {
    info!("Upserting a bulk of {} messages…", messages.len());
    let mut connection = db.connection()?;
    let transaction = connection.transaction()?;

    for message in messages.iter() {
        if message.type_ == MessageType::ReadLogged {
            debug!("[{:?}] {}", &message.type_, &message.sensor.id);
            message.upsert_into(&*transaction)?;
        }
    }

    transaction.commit()?;
    Ok(())
}
