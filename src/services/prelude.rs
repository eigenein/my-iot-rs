use std::time::Duration;

pub use crate::services::helpers::expect::expect;
pub use crate::services::helpers::handle_result::handle_service_result;
pub use crate::services::helpers::{call_json_api, deserialize_timestamp, CLIENT};

pub const MINUTE: Duration = Duration::from_secs(60);
