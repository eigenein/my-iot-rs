use std::str::FromStr;

use sentry::types::Dsn;
use sentry::{ClientInitGuard, ClientOptions};

use crate::prelude::*;

/// Initialize Sentry integration.
pub fn init(dsn: impl AsRef<str>) -> ClientInitGuard {
    sentry::init(ClientOptions {
        dsn: Some(Dsn::from_str(dsn.as_ref()).unwrap()),
        release: Some(crate_version!().into()),
        ..Default::default()
    })
}
