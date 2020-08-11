use crate::prelude::*;

pub async fn handle_service_result(service_id: &str, sleep_interval: Duration, result: Result) {
    if let Err(error) = result {
        error!("[{}] {}", service_id, error);
    }
    task::sleep(sleep_interval).await;
}
