pub use reqwest::blocking::Client;
use reqwest::blocking::ClientBuilder;
use reqwest::header::{HeaderMap, HeaderValue};
use std::time::Duration;
use structopt::clap::crate_version;

pub const USER_AGENT: &str = concat!(
    "My IoT / ",
    crate_version!(),
    " (Rust; https://github.com/eigenein/my-iot-rs)"
);

pub fn client_builder() -> ClientBuilder {
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_static(USER_AGENT));
    Client::builder()
        .gzip(true)
        .use_rustls_tls()
        .default_headers(headers)
        .timeout(Duration::from_secs(10))
}

pub fn default_client() -> Client {
    client_builder().build().expect("Failed to instantiate an HTTP client")
}
