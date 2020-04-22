use crate::prelude::*;

#[derive(Debug)]
pub struct InternalError {
    pub description: String,
}

impl InternalError {
    pub fn new<S: Into<String>>(description: S) -> Self {
        InternalError {
            description: description.into(),
        }
    }
}

impl Error for InternalError {}

impl std::fmt::Display for InternalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.description)
    }
}
