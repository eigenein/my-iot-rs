//! Application error and conversions from the other errors.

use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct Error {
    message: String,
}

impl Error {
    pub fn new<M: Into<String>>(message: M) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str(&self.message)
    }
}

impl From<&str> for Error {
    fn from(message: &str) -> Self {
        Self::new(message.to_string())
    }
}

impl From<String> for Error {
    fn from(message: String) -> Self {
        Self::new(message)
    }
}

macro_rules! from_error {
    ($type_:ty, $message:expr) => {
        impl From<$type_> for Error {
            fn from(error: $type_) -> Self {
                Self::new(format!("{}: {}", $message, error.to_string()))
            }
        }
    };
}

from_error!(reqwest::Error, "Request error");
from_error!(rhai::ParseError, "Rhai parsing has failed");
from_error!(rocket::config::ConfigError, "Rocket configuration error");
from_error!(rocket::error::LaunchError, "Rocket launch error");
from_error!(rusqlite::Error, "SQLite error");
from_error!(spa::SpaError, "Sunrise/sunset calculation error");
from_error!(std::boxed::Box<bincode::ErrorKind>, "Bincode serialization error");
from_error!(std::boxed::Box<rhai::EvalAltResult>, "Rhai evaluation error");
from_error!(std::io::Error, "I/O error");
from_error!(toml::de::Error, "TOML deserialization error");
from_error!(url::ParseError, "URL parse error");
from_error!(simplelog::TermLogError, "Logging initialization error");
from_error!(toml::ser::Error, "TOML serialization error");
from_error!(std::num::ParseFloatError, "Number could not be parsed");
from_error!(std::num::TryFromIntError, "Number could not be converted");
from_error!(serde_json::Error, "JSON error");
