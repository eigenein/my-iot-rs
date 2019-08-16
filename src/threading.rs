//! Threading utilities.

use std::sync::{Arc, Mutex};
use std::thread;

pub type ArcMutex<T> = Arc<Mutex<T>>;
pub type JoinHandle = thread::JoinHandle<()>;

/// Convenience function to spawn a named thread.
pub fn spawn<F, T>(name: String, f: F) -> thread::JoinHandle<T>
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
{
    thread::Builder::new().name(name).spawn(f).unwrap()
}
