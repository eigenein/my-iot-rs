use futures::future::BoxFuture;
use surf::{http::headers, middleware::Next, Request, Response};

use crate::prelude::*;

pub const REQUEST_TIMEOUT_SECS: u64 = 60;
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(REQUEST_TIMEOUT_SECS);

/// Middleware that injects the default HTTP headers to all outgoing requests.
pub fn inject_default_headers(request: Request, client: Client, next: Next) -> BoxFuture<surf::Result<Response>> {
    pub const USER_AGENT: &str = concat!(
        "My IoT / ",
        crate_version!(),
        " (Rust; https://github.com/eigenein/my-iot-rs)"
    );

    Box::pin(async move {
        let mut request = request;
        request.insert_header(headers::USER_AGENT, USER_AGENT);
        Ok(next.run(request, client).await?)
    })
}

pub fn timeout_request(request: Request, client: Client, next: Next) -> BoxFuture<surf::Result<Response>> {
    Box::pin(async move { async_std::future::timeout(REQUEST_TIMEOUT, next.run(request, client)).await? })
}
