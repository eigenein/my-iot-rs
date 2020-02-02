//! # Message bus
//!
//! The bus implements many-producer-many-consumer queue and allows each service
//! to listen to each other service.

use crate::prelude::*;

pub struct Bus {
    /// Service message inbox senders.
    service_txs: Vec<Sender<Message>>,

    /// The bus message inbox sender.
    tx: Sender<Message>,

    /// The bus message inbox receiver.
    rx: Receiver<Message>,
}

impl Bus {
    pub fn new() -> Self {
        let (tx, rx) = crossbeam_channel::unbounded::<Message>();
        Self {
            tx,
            rx,
            service_txs: Vec::new(),
        }
    }

    /// Get a new message sender. Essentially, it makes a clone of the bus inbox.
    pub fn add_tx(&self) -> Sender<Message> {
        self.tx.clone()
    }

    /// Get a new receiver to subscribe to the bus.
    pub fn add_rx(&mut self) -> Receiver<Message> {
        let (tx, rx) = crossbeam_channel::unbounded();
        self.service_txs.push(tx);
        rx
    }

    /// Spawn the bus dispatcher thread.
    pub fn spawn(self) -> Result<()> {
        info!("Spawning message bus…");
        supervisor::spawn("my-iot::bus", self.tx.clone(), move || -> Result<()> {
            for message in &self.rx {
                debug!("Dispatching {}", &message.sensor.sensor_id);
                for tx in self.service_txs.iter() {
                    if let Err(error) = tx.send(message.clone()) {
                        error!("Could not send the message to {:?}: {:?}", tx, error);
                    }
                }
                debug!("Dispatched {}", &message.sensor.sensor_id);
            }
            Err(format_err!("Receiver channel is unexpectedly exhausted"))
        })?;
        Ok(())
    }
}
