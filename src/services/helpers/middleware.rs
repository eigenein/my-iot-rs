use futures::future::BoxFuture;
use surf::{http::headers, middleware::Next, Request, Response};

use crate::prelude::*;

pub const REQUEST_TIMEOUT_SECS: u64 = 60;
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(REQUEST_TIMEOUT_SECS);

/// Injects the default HTTP headers to all outgoing requests.
pub fn inject_default_headers(request: Request, client: Client, next: Next) -> BoxFuture<surf::Result<Response>> {
    pub const USER_AGENT: &str = concat!(
        "My IoT / ",
        crate_version!(),
        " (Rust; https://github.com/eigenein/my-iot-rs)"
    );

    Box::pin(async move {
        let mut request = request;
        request.insert_header(headers::USER_AGENT, USER_AGENT);
        next.run(request, client).await
    })
}

/// Sets the timeout on the request.
pub fn timeout_request(request: Request, client: Client, next: Next) -> BoxFuture<surf::Result<Response>> {
    Box::pin(async move { async_std::future::timeout(REQUEST_TIMEOUT, next.run(request, client)).await? })
}

/// Converts client and server errors into [`surf::Error`].
pub fn error_for_status(request: Request, client: Client, next: Next) -> BoxFuture<surf::Result<Response>> {
    Box::pin(async move {
        let mut response = next.run(request, client).await?;
        let status = response.status();
        if status.is_client_error() || status.is_server_error() {
            Err(surf::Error::from_str(
                status,
                response
                    .body_string()
                    .await
                    .unwrap_or_else(|_| status.canonical_reason().to_string()),
            ))
        } else {
            Ok(response)
        }
    })
}
