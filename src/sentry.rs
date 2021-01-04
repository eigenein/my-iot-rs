use sentry::ClientInitGuard;

/// Initialize Sentry integration.
pub fn init(dsn: impl AsRef<str>) -> ClientInitGuard {
    sentry::init(dsn.as_ref())
}
