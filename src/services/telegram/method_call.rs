use crate::prelude::*;
use crate::services::telegram::TelegramChatId;

#[derive(Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum MethodCall {
    /// https://core.telegram.org/bots/api#getupdates
    GetUpdates {
        #[serde(skip_serializing_if = "Option::is_none")]
        offset: Option<i64>,

        timeout: u64,

        allowed_updates: &'static [&'static str],
    },

    /// <https://core.telegram.org/bots/api#sendmessage>
    SendMessage {
        chat_id: TelegramChatId,
        text: String,

        /// <https://core.telegram.org/bots/api#formatting-options>
        #[serde(skip_serializing_if = "Option::is_none")]
        parse_mode: Option<String>,
    },

    /// <https://core.telegram.org/bots/api#sendvideo>
    SendVideo {
        chat_id: TelegramChatId,

        /// Only allows to pass a URL or a file ID.
        /// Use `input_file` parameter to send a `Bytes`.
        #[serde(skip_serializing_if = "Option::is_none")]
        video: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        caption: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        parse_mode: Option<String>,
    },
}

impl MethodCall {
    /// Gets the URL method part.
    pub fn url_part(&self) -> &'static str {
        match self {
            MethodCall::GetUpdates { .. } => "getUpdates",
            MethodCall::SendMessage { .. } => "sendMessage",
            MethodCall::SendVideo { .. } => "sendVideo",
        }
    }
}
