use lazy_static::lazy_static;
use reqwest::Method;

use crate::prelude::*;
use reqwest::header::{HeaderMap, HeaderValue};

pub mod expect;
pub mod handle_result;

lazy_static! {
    /// `Client` instance used to make requests to all services.
    pub static ref CLIENT: Client = build_client().expect("Failed to build a client");
}

/// Builds an HTTP client to use with a service.
fn build_client() -> Result<Client> {
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_static(USER_AGENT));
    Ok(Client::builder()
        .gzip(true)
        .use_rustls_tls()
        .default_headers(headers)
        .timeout(Duration::from_secs(30))
        .pool_idle_timeout(Some(Duration::from_secs(300)))
        .build()?)
}

/// Deserializes a Unix time into `DateTime<Local>`.
pub fn deserialize_timestamp<'de, D: Deserializer<'de>>(deserializer: D) -> Result<DateTime<Local>, D::Error> {
    Ok(Local.timestamp(i64::deserialize(deserializer)?, 0))
}

/// Generic function to call a JSON API.
pub async fn call_json_api<U, R>(method: Method, access_token: &str, url: U) -> Result<R>
where
    U: AsRef<str> + std::fmt::Display,
    R: DeserializeOwned,
{
    debug!("Calling {}…", url);
    let response = CLIENT
        .request(method, url.as_ref())
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    debug!("Finished {}.", url);
    Ok(response)
}
