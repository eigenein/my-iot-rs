//! # Message bus
//!
//! The bus implements many-producer-many-consumer queue and allows each service
//! to listen to each other service.

use std::sync::atomic::{AtomicU64, Ordering};

use crate::prelude::*;

pub struct Bus {
    /// Service message inbox senders.
    service_txs: Vec<Sender>,

    /// The bus message inbox sender.
    tx: Sender,

    /// The bus message inbox receiver.
    rx: Receiver,

    message_counter: Arc<AtomicU64>,
}

impl Bus {
    pub fn new(message_counter: Arc<AtomicU64>) -> Self {
        let (tx, rx) = crossbeam::channel::unbounded::<Message>();
        Self {
            tx,
            rx,
            message_counter,
            service_txs: Vec::new(),
        }
    }

    /// Get a new message sender. Essentially, it makes a clone of the bus inbox.
    pub fn add_tx(&self) -> Sender {
        self.tx.clone()
    }

    /// Get a new receiver to subscribe to the bus.
    pub fn add_rx(&mut self) -> Receiver {
        let (tx, rx) = crossbeam::channel::unbounded();
        self.service_txs.push(tx);
        rx
    }

    /// Spawn the bus dispatcher thread.
    pub fn spawn(self) -> Result {
        info!("Spawning message bus…");
        thread::Builder::new().name("system::bus".into()).spawn(move || {
            for message in &self.rx {
                Self::log_message(&message);
                for tx in self.service_txs.iter() {
                    message.clone().send_and_forget(&tx);
                }
                let number = self.message_counter.fetch_add(1, Ordering::Relaxed);
                debug!("Dispatched (#{}) {}", number, &message.sensor.id);
            }
            unreachable!();
        })?;
        Ok(())
    }

    fn log_message(message: &Message) {
        match &message.reading.value {
            Value::Blob(content) => info!("[{:?}] {}: {} bytes", &message.type_, &message.sensor.id, content.len()),
            ref value => info!("[{:?}] {} = {:?}", &message.type_, &message.sensor.id, value),
        }
    }
}

impl Message {
    /// Send the message via the specified sender and log and ignore any errors.
    pub fn send_and_forget(self, tx: &Sender) {
        if let Err(error) = tx.send(self) {
            debug!("Could not send the message: {}", error.to_string());
        }
    }
}
