//! Allows to monitor thread status and automatically respawn a crashed thread.

use crate::Result;
use std::thread;

/// Spawn a supervised named thread.
pub fn spawn<N, F>(name: N, f: F) -> std::io::Result<()>
where
    N: Into<String>,
    F: Fn() -> Result<()> + Send + 'static,
{
    thread::Builder::new().name(name.into()).spawn(f)?;
    Ok(())
}
