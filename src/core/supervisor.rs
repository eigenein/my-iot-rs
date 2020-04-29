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
    let sensor = format!("my_iot::{}::is_running", &name);

    thread::Builder::new().name(name.clone()).spawn(move || loop {
        info!("Running `{}`", &name);
        Composer::new(&sensor)
            .value(Value::Boolean(true))
            .room_title("System".to_string())
            .message
            .send_and_forget(&tx);
        match f() {
            Ok(_) => error!("Thread `{}` has finished unexpectedly", &name),
            Err(error) => error!("Thread `{}` crashed: {:?}", &name, error),
        }
        Composer::new(&sensor)
            .value(Value::Boolean(false))
            .room_title("System")
            .message
            .send_and_forget(&tx);

        // FIXME: https://github.com/eigenein/my-iot-rs/issues/47
        thread::sleep(Duration::from_secs(60));
    })?;

    Ok(())
}
