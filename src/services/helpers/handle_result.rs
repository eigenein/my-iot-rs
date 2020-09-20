use crate::prelude::*;

/// Log the result and sleep for the specified duration.
pub async fn handle_service_result(service_id: &str, sleep_duration: Duration, result: Result) {
    let _ = result.log(|| format!("[{}]", service_id));
    task::sleep(sleep_duration).await;
}
