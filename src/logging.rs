//! Logging setup.

use env_logger::DEFAULT_FILTER_ENV;
use std::env::var;

/// Initialize logging.
pub fn init() {
    pretty_env_logger::formatted_timed_builder()
        .parse_filters(&var(DEFAULT_FILTER_ENV).unwrap_or_else(|_| "info".into()))
        .init();
}
