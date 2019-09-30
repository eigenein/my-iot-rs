//! Allows to monitor thread status and automatically respawn a crashed thread.

use crate::Result;
use log::{error, info};
use std::{fmt, thread};

/// Spawn a supervised named thread.
pub fn spawn<N, F>(name: N, f: F) -> std::io::Result<()>
where
    N: AsRef<str> + Send + fmt::Display + 'static,
    F: Fn() -> Result<()> + Send + 'static,
{
    thread::Builder::new().name(name.as_ref().into()).spawn(move || loop {
        info!("Running {}", name);
        match f() {
            Ok(_) => error!("Thread {} has finished unexpectedly", name.as_ref()),
            Err(error) => error!("Thread {} crashed: {:?}", name.as_ref(), error),
        }
    })?;
    Ok(())
}
