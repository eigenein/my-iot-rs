use crate::prelude::*;

use crate::services::helpers::middleware::{error_for_status, inject_default_headers, timeout_request};
use lazy_static::lazy_static;
use surf::middleware::Redirect;

lazy_static! {
    /// `Client` instance used to make outgoing requests.
    pub static ref CLIENT: Client = surf::client()
        .with(Redirect::default())
        .with(inject_default_headers)
        .with(timeout_request)
        .with(error_for_status);
}
