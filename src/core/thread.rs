use crate::prelude::*;
use std::time::Duration;

/// Spawns a service thread which just periodically invokes the `loop_` function.
/// This is a frequently repeated pattern in the services.
pub fn spawn_service_loop<F>(service_id: String, interval: Duration, loop_: F) -> Result
where
    F: Fn() -> Result,
    F: Send + 'static,
{
    thread::spawn(move || loop {
        if let Err(error) = loop_() {
            error!("[{}] The iteration has failed: {}", service_id, error.to_string());
        }
        thread::sleep(interval);
    });
    Ok(())
}
