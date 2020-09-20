pub use std::borrow::Borrow;
pub use std::collections::HashMap;
pub use std::convert::{TryFrom, TryInto};
pub use std::future::Future;
pub use std::time::{Duration, Instant};

pub use anyhow::anyhow;
pub use async_std::{
    sync::{Arc, Mutex, MutexGuard},
    task,
};
pub use bytes::Bytes;
pub use chrono::prelude::*;
pub use futures::prelude::*;
pub use log::{debug, error, info, log, warn, Level as LogLevel};
pub use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize};
pub use structopt::clap::crate_version;
pub use surf::Client;

pub use crate::core::bus::Bus;
pub use crate::core::db::{reading::Reading, sensor::Sensor, Connection};
pub use crate::core::message::{Message, Type as MessageType};
pub use crate::core::si::*;
pub use crate::core::value::{from::*, try_into::*, *};
pub use crate::logging::Log;

pub type Result<T = ()> = anyhow::Result<T>;
pub type StdResult<T, E> = std::result::Result<T, E>;
pub type JoinHandle = async_std::task::JoinHandle<Result>;
pub type Receiver = futures::channel::mpsc::UnboundedReceiver<Message>;
pub type Sender = futures::channel::mpsc::UnboundedSender<Message>;

/// Converts the object into its debug representation.
pub fn to_debug_string<T: std::fmt::Debug>(this: &mut T) -> String {
    format!("{:?}", this)
}
