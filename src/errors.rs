use crate::prelude::*;

#[derive(Debug)]
pub struct InternalError(pub String);

impl Error for InternalError {}

impl std::fmt::Display for InternalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
