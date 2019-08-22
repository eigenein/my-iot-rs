//! Threading utilities.

use std::sync::{Arc, Mutex};
use std::thread;

pub type ArcMutex<T> = Arc<Mutex<T>>;

/// Convenience function to spawn a named thread.
pub fn spawn<N, F, T>(name: N, f: F) -> std::io::Result<thread::JoinHandle<T>>
where
    N: Into<String>,
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
{
    thread::Builder::new().name(name.into()).spawn(f)
}
