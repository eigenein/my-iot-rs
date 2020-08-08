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
        thread::spawn(move || {
            for message in &rx {
                buffer.lock().unwrap().push(message);
            }
            unreachable!();
        });
    }

    thread::spawn(move || loop {
        sleep(Duration::from_millis(COMMIT_INTERVAL_MILLIS));

        // Acquire the lock, drain the buffer and release the lock immediately.
        let messages: Vec<Message> = { buffer.lock().unwrap().drain(..).collect() };

        if !messages.is_empty() {
            let start_time = Instant::now();
            if let Err(error) = upsert_messages(&db, messages) {
                error!("could not upsert the messages: {}", error);
            }
            info!("Took {:.1?}.", start_time.elapsed());
        }
    });

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
