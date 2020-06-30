use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Holds a number of handled `Message`s.
pub struct MessageCounter(pub Arc<AtomicU64>);

impl MessageCounter {
    /// Extracts the inner value.
    pub fn value(&self) -> u64 {
        self.0.load(Ordering::Relaxed)
    }
}
