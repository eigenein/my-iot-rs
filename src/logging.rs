//! Logging setup.

/// Initialize logging.
pub fn init() {
    // Set info level by default, if not specified.
    std::env::set_var(
        "RUST_LOG",
        std::env::var("RUST_LOG").unwrap_or_else(|_| String::from("info")),
    );
    pretty_env_logger::init_timed();
}
