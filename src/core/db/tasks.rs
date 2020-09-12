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
            let messages: Vec<Message> = {
                buffer
                    .lock()
                    .await
                    .drain(..)
                    .filter(|message| message.type_ == MessageType::ReadLogged)
                    .collect()
            };

            if !messages.is_empty() {
                let start_time = Instant::now();
                if let Err(error) = db.upsert_messages(messages).await {
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
        unreachable!();
    });
}
