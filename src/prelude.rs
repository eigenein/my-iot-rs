pub use crate::core::message::{Composer, Message, Type as MessageType};
pub use crate::core::persistence::models::*;
pub use crate::core::value::{PointOfTheCompass, Value};
pub use crossbeam_channel::Sender;
pub use failure::{format_err, Error};
pub use log::{debug, error, info, warn};
pub use std::sync::{Arc, Mutex};

pub type Result<T> = std::result::Result<T, Error>;
