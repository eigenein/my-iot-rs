//! Defines a structure and the related operations to implement a min-heap
//! with the max-heap from the standard library.

use chrono::{DateTime, Local};
use std::cmp::Ordering;

/// Min-heap reading entry.
pub struct MinHeapReading(pub DateTime<Local>, pub f64);

impl PartialEq for MinHeapReading {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for MinHeapReading {}

impl PartialOrd for MinHeapReading {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MinHeapReading {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reversed ordering for the min-heap.
        other.0.cmp(&self.0)
    }
}
