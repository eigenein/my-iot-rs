//! Type aliases.

use std::sync::{Arc, Mutex};

pub type ArcMutex<T> = Arc<Mutex<T>>;
