//! Database persistence thread.

use crate::prelude::*;

// TODO: make configurable.
const COMMIT_INTERVAL_MILLIS: u64 = 1000;

/// Spawn the persistence thread.
pub fn spawn(db: Connection, bus: &mut Bus) {
    info!("Spawning persistence tasks…");
    let buffer = Arc::new(Mutex::new(Vec::<Message>::new()));

    spawn_committer(db, buffer.clone());
    spawn_bufferizer(bus.add_rx(), buffer);
}

/// Spawns the task that periodically commits the buffered messages.
fn spawn_committer(db: Connection, buffer: Arc<Mutex<Vec<Message>>>) {
    task::spawn(async move {
        loop {
            task::sleep(Duration::from_millis(COMMIT_INTERVAL_MILLIS)).await;

            // Acquire the lock, drain the buffer and release the lock immediately.
            let messages: Vec<Message> = { buffer.lock().await.drain(..).collect() };

            if !messages.is_empty() {
                let start_time = Instant::now();
                if let Err(error) = upsert_messages(&db, messages).await {
                    error!("could not upsert the messages: {}", error);
                }
                info!("Took {:.1?}.", start_time.elapsed());
            }
        }
    });
}

/// Spawns the task that bufferizes the messages from the MPMC queue.
fn spawn_bufferizer(mut rx: Receiver, buffer: Arc<Mutex<Vec<Message>>>) {
    task::spawn(async move {
        while let Some(message) = rx.next().await {
            buffer.lock().await.push(message);
        }
        Err::<(), _>(Error::new("The buffering task has stopped"))
    });
}

/// Upserts the messages within a single transaction.
///
/// Inserting messages one by one is quite slow on low-performance boards.
/// Thus, I spin up a separate thread which accumulates incoming messages
/// and periodically upserts them all within a single transaction.
async fn upsert_messages(db: &Connection, messages: Vec<Message>) -> Result {
    info!("Upserting a bulk of {} messages…", messages.len());
    let mut connection = db.connection().await;
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
