pub use crate::core::bus::Bus;
pub use crate::core::message::{Message, Type as MessageType};
pub use crate::core::persistence::reading::Reading;
pub use crate::core::persistence::sensor::Sensor;
pub use crate::core::persistence::Connection;
pub use crate::core::value::{PointOfTheCompass, Value};
pub use chrono::prelude::*;
pub use crossbeam::thread::Scope;
pub use log::{debug, error, info, log, warn, Level as LogLevel};
pub use serde::{Deserialize, Deserializer, Serialize};
pub use std::collections::HashMap;
pub use std::error::Error;
pub use std::result::Result as StdResult;
pub use std::thread;
pub use std::thread::sleep;

pub type Result<T> = StdResult<T, Box<dyn Error>>;
pub type Receiver = crossbeam::channel::Receiver<Message>;
pub type Sender = crossbeam::channel::Sender<Message>;
