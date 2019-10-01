//! Allows to monitor thread status and automatically respawn a crashed thread.

use crate::message::{Message, Type};
use crate::Result;
use crossbeam_channel::Sender;
use log::{error, info};
use std::time::Duration;
use std::{io, thread};

/// Spawn a supervised named thread.
pub fn spawn<N, F>(name: N, tx: Sender<Message>, f: F) -> io::Result<()>
where
    N: Into<String>,
    F: Fn() -> Result<()> + Send + 'static,
{
    let name = name.into();
    let sensor = format!("{}::is_running", &name);

    thread::Builder::new().name(name.clone()).spawn(move || loop {
        // TODO: update thread status.
        info!("Running {}", &name);
        tx.send(Message::now(Type::Actual, &sensor, true)).unwrap();
        match f() {
            Ok(_) => error!("Thread {} has finished unexpectedly", &name),
            Err(error) => error!("Thread {} crashed: {:?}", &name, error),
        }
        tx.send(Message::now(Type::Actual, &sensor, false)).unwrap();

        // FIXME
        thread::sleep(Duration::from_secs(60));
    })?;

    Ok(())
}
