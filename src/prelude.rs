pub use std::collections::HashMap;
pub use std::error::Error;
pub use std::thread;
pub use std::thread::sleep;

pub use chrono::prelude::*;
pub use log::{debug, error, info, log, warn, Level as LogLevel};
pub use reqwest::blocking::Client;
pub use serde::de::DeserializeOwned;
pub use serde::{Deserialize, Deserializer, Serialize};
pub use structopt::clap::crate_version;

pub use crate::core::bus::Bus;
pub use crate::core::message::{Message, Type as MessageType};
pub use crate::core::persistence::reading::Reading;
pub use crate::core::persistence::sensor::Sensor;
pub use crate::core::persistence::Connection;
pub use crate::core::value::{PointOfTheCompass, Value};

pub type Result<T, E = Box<dyn Error>> = std::result::Result<T, E>;
pub type Receiver = crossbeam::channel::Receiver<Message>;
pub type Sender = crossbeam::channel::Sender<Message>;

/// Amount of [Joule](https://en.wikipedia.org/wiki/Joule)s
/// in 1 [watt-hour](https://en.wikipedia.org/wiki/Kilowatt-hour).
pub const JOULES_IN_WH: f64 = 3600.0;

pub const WH_IN_JOULE: f64 = 1.0 / JOULES_IN_WH;

/// `User-Agent` header used for all outcoming HTTP requests.
pub const USER_AGENT: &str = concat!(
    "My IoT / ",
    crate_version!(),
    " (Rust; https://github.com/eigenein/my-iot-rs)"
);

/// Converts the object into its debug representation.
pub fn to_debug_string<T: std::fmt::Debug>(this: &mut T) -> String {
    format!("{:?}", this)
}

pub fn to_string<O: ToString>(object: O) -> String {
    object.to_string()
}
