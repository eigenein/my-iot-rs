//! Allows to monitor thread status and automatically respawn a crashed thread.

use crate::prelude::*;
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
        // FIXME: dangerous `unwrap`s.
        tx.send(Composer::new(&sensor).value(Value::Boolean(true)).into())
            .unwrap();
        match f() {
            Ok(_) => error!("Thread {} has finished unexpectedly", &name),
            Err(error) => error!("Thread {} crashed: {:?}", &name, error),
        }
        tx.send(Composer::new(&sensor).value(Value::Boolean(false)).into())
            .unwrap();

        // FIXME: https://github.com/eigenein/my-iot-rs/issues/47
        thread::sleep(Duration::from_secs(60));
    })?;

    Ok(())
}
