//! Database persistence tasks.

use crate::prelude::*;

const COMMIT_INTERVAL_MIN: Duration = Duration::from_millis(50);
const COMMIT_INTERVAL_MAX: Duration = Duration::from_millis(5000);

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
        let mut commit_interval = Duration::from_millis(1000);

        loop {
            task::sleep(commit_interval).await;

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
                info!("Upserting a bulk of {} messages…", messages.len());
                let start_time = Instant::now();
                let _ = db.upsert_messages(messages).await.log(|| "failed to upsert");
                let elapsed = start_time.elapsed();
                info!("Upserted in {:.1?}.", elapsed);

                if elapsed > commit_interval {
                    commit_interval = COMMIT_INTERVAL_MAX.min(commit_interval * 2);
                } else if elapsed < commit_interval / 2 {
                    commit_interval = COMMIT_INTERVAL_MIN.max(commit_interval / 2);
                }
                info!("Commit interval: {:?}.", commit_interval);
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
