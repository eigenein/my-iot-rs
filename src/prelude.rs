pub use crate::core::db::Db;
pub use crate::core::message::{Composer, Message, Type as MessageType};
pub use crate::core::value::{PointOfTheCompass, Value};
pub use failure::{format_err, Error};
pub use log::{debug, info, warn};

pub type Result<T> = std::result::Result<T, Error>;
