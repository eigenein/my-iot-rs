//! Allows monitoring thread status and automatically re-spawning a crashed thread.

use crate::prelude::*;
use log::{error, info};
use std::time::Duration;
use std::{process, thread};

/// Spawn a supervised named thread.
pub fn spawn<'env, F>(scope: &Scope<'env>, name: &'env str, tx: Sender<Message>, f: F) -> Result<()>
where
    F: Fn() -> Result<()> + Send + 'env,
{
    let sensor = format!("my_iot::{}::is_running", name);

    scope.builder().name(name.to_string()).spawn(move |_| loop {
        info!("[{}] Running `{}`", process::id(), &name);
        Message::new(&sensor)
            .value(Value::Boolean(true))
            .room_title("System".to_string())
            .sensor_title(format!("Running {}", name))
            .send_and_forget(&tx);
        match f() {
            Ok(_) => error!("[{}] Thread `{}` has finished unexpectedly", process::id(), &name),
            Err(error) => error!("[{}] Thread `{}` crashed: {}", process::id(), &name, error.to_string()),
        }
        Message::new(&sensor)
            .value(Value::Boolean(false))
            .room_title("System")
            .sensor_title(format!("Running {}", name))
            .send_and_forget(&tx);

        // FIXME: https://github.com/eigenein/my-iot-rs/issues/47
        thread::sleep(Duration::from_secs(60));
    })?;

    Ok(())
}
