use std::time::Duration;

pub use crate::services::helpers::client::CLIENT;
pub use crate::services::helpers::deserialize_timestamp;
pub use crate::services::helpers::expect::expect;
pub use crate::services::helpers::handle_result::handle_service_result;
pub use crate::services::helpers::middleware::inject_default_headers;

pub const MINUTE: Duration = Duration::from_secs(60);
