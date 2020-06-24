use crate::prelude::*;

#[derive(PartialEq, Debug, Clone, Serialize)]
pub struct Reading {
    /// Timestamp when the value has been actually measured.
    /// This may be earlier than a moment of sending the message.
    pub timestamp: DateTime<Local>,

    /// Read sensor value.
    pub value: Value,
}
