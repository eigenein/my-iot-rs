use crate::prelude::*;

/// Spawn message dispatcher that broadcasts every received message to emulate
/// a multi-producer multi-consumer queue.
///
/// Thus, services exchange messages with each other. Each message from the input channel is
/// broadcasted to each of output channels.
///
/// - `rx`: dispatcher input message channel
/// - `tx`: dispatcher output message channel
/// - `txs`: service output message channels
pub fn spawn(rx: Receiver<Message>, tx: Sender<Message>, txs: Vec<Sender<Message>>) -> Result<()> {
    info!("Spawning message dispatcher…");
    supervisor::spawn("my-iot::dispatcher", tx, move || -> Result<()> {
        for message in &rx {
            debug!("Dispatching {}", &message.sensor.sensor_id);
            for tx in txs.iter() {
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
