//! Logging setup.

/// Init logging.
pub fn init() {
    std::env::set_var(
        "RUST_LOG",
        std::env::var("RUST_LOG").unwrap_or_else(|_| String::from("info")),
    );
    pretty_env_logger::init();
}
