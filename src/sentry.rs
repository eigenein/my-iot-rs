use sentry::integrations::log::LogIntegration;
use sentry::{ClientInitGuard, ClientOptions};

/// Initialize Sentry integration.
pub fn init(dsn: impl AsRef<str>) -> ClientInitGuard {
    sentry::init(
        Into::<ClientOptions>::into(dsn.as_ref()).add_integration(LogIntegration {
            emit_warning_events: true,
            ..Default::default()
        }),
    )
}
