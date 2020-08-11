//! # Message bus
//!
//! The bus implements many-producer-many-consumer queue and allows each service
//! to listen to each other service.

use crate::prelude::*;

pub struct Bus {
    /// Service message inbox senders.
    consumers: Vec<Sender>,

    /// The bus message inbox sender.
    producer_tx: Sender,

    /// The bus message inbox receiver.
    producer_rx: Receiver,
}

impl Bus {
    pub fn new() -> Self {
        let (tx, rx) = futures::channel::mpsc::unbounded();
        Self {
            producer_tx: tx,
            producer_rx: rx,
            consumers: Vec::new(),
        }
    }

    /// Get a new message sender. Essentially, it makes a clone of the bus inbox.
    pub fn add_tx(&self) -> Sender {
        self.producer_tx.clone()
    }

    /// Get a new receiver to subscribe to the bus.
    pub fn add_rx(&mut self) -> Receiver {
        let (tx, rx) = futures::channel::mpsc::unbounded();
        self.consumers.push(tx);
        rx
    }

    /// Spawn the bus dispatcher thread.
    pub fn spawn(mut self) -> JoinHandle {
        info!("Spawning message bus…");
        task::spawn(async move {
            while let Some(message) = self.producer_rx.next().await {
                Self::log_message(&message);
                for tx in self.consumers.iter_mut() {
                    message.clone().send_to(tx).await;
                }
            }
            unreachable!();
        })
    }

    fn log_message(message: &Message) {
        match &message.reading.value {
            Value::Blob(content) => info!("[{:?}] {}: {} bytes", &message.type_, &message.sensor.id, content.len()),
            ref value => info!("[{:?}] {} = {:?}", &message.type_, &message.sensor.id, value),
        }
    }
}
