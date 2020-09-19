use crate::prelude::*;

pub mod client;
pub mod expect;
pub mod handle_result;
pub mod middleware;

/// Deserializes a Unix time into `DateTime<Local>`.
pub fn deserialize_timestamp<'de, D: Deserializer<'de>>(deserializer: D) -> StdResult<DateTime<Local>, D::Error> {
    Ok(Local.timestamp(i64::deserialize(deserializer)?, 0))
}
