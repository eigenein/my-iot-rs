//! [Telegram bot](https://core.telegram.org/bots/api) service which is able to receive and send messages.

use std::fmt::Debug;
use std::time::Duration;

use log::debug;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::prelude::*;
use crate::services::CLIENT;
use std::any::TypeId;

const GET_UPDATES_TIMEOUT_SECS: u64 = 60;

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Telegram {
    secrets: Secrets,
}

/// Secrets section.
#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Secrets {
    token: String,
}

impl Telegram {
    pub fn spawn(self, service_id: String, bus: &mut Bus) -> Result<()> {
        let tx = bus.add_tx();

        thread::Builder::new().name(service_id.clone()).spawn(move || {
            let mut offset: Option<i64> = None;
            loop {
                match self.loop_(&service_id, offset, &tx) {
                    Ok(new_offset) => offset = new_offset,
                    Err(error) => {
                        error!("Failed to refresh the sensors: {}", error.to_string());
                        sleep(Duration::from_secs(60));
                    }
                }
            }
        })?;

        Ok(())
    }

    fn loop_(&self, service_id: &str, offset: Option<i64>, tx: &Sender) -> Result<Option<i64>> {
        let mut offset = offset;
        for update in self.get_updates(offset)?.iter() {
            offset = offset.max(Some(update.update_id + 1));
            self.send_readings(&service_id, &tx, &update)?;
        }
        debug!("{}: next offset: {:?}", &service_id, offset);
        Ok(offset)
    }

    /// Send reading messages from the provided Telegram update.
    fn send_readings(&self, service_id: &str, tx: &Sender, update: &TelegramUpdate) -> Result<()> {
        debug!("{}: {:?}", service_id, &update);

        if let Some(ref message) = update.message {
            if let Some(ref text) = message.text {
                tx.send(
                    Message::new(format!("{}::{}::message", service_id, message.chat.id))
                        .type_(MessageType::ReadNonLogged)
                        .value(Value::Text(text.into()))
                        .timestamp(message.date),
                )?;
            }
        }

        Ok(())
    }

    /// <https://core.telegram.org/bots/api#getupdates>
    fn get_updates(&self, offset: Option<i64>) -> Result<Vec<TelegramUpdate>> {
        self.call(&TelegramGetUpdates {
            offset,
            timeout: GET_UPDATES_TIMEOUT_SECS,
            allowed_updates: &["message"],
        })
    }

    /// Calls a [Telegram Bot API](https://core.telegram.org/bots/api) method.
    pub fn call<P: TelegramRequest + 'static, R>(&self, request: &P) -> Result<R>
    where
        P: Serialize + Debug + ?Sized,
        R: DeserializeOwned,
    {
        let method = request.method();
        debug!("{}({:?})", method, request);
        // FIXME: https://github.com/eigenein/my-iot-rs/issues/44
        let mut request = CLIENT
            .get(&format!(
                "https://api.telegram.org/bot{}/{}",
                self.secrets.token, method,
            ))
            .json(request);
        if TypeId::of::<P>() == TypeId::of::<TelegramGetUpdates>() {
            request = request.timeout(Duration::from_secs(GET_UPDATES_TIMEOUT_SECS + 1));
        }
        match request.send()?.error_for_status()?.json::<TelegramResponse<R>>()? {
            TelegramResponse::Result { result } => Ok(result),
            TelegramResponse::Error { description } => {
                error!("Telegram error: {:?}", description);
                Err(description.into())
            }
        }
    }
}

/// <https://core.telegram.org/bots/api#making-requests>
#[derive(Deserialize)]
#[serde(untagged)]
pub enum TelegramResponse<T> {
    Result { result: T },
    Error { description: String },
}

#[derive(Deserialize, Debug)]
pub struct TelegramUpdate {
    pub update_id: i64,
    pub message: Option<TelegramMessage>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum TelegramChatId {
    UniqueId(i64),
    Username(String),
}

/// <https://core.telegram.org/bots/api#message>
#[derive(Deserialize, Debug)]
pub struct TelegramMessage {
    pub message_id: i64,

    #[serde(deserialize_with = "chrono::serde::ts_seconds::deserialize")]
    pub date: DateTime<Utc>,

    pub chat: TelegramChat,
    pub text: Option<String>,
}

/// <https://core.telegram.org/bots/api#chat>
#[derive(Deserialize, Debug)]
pub struct TelegramChat {
    pub id: i64,
}

/// <https://core.telegram.org/bots/api#formatting-options>
#[derive(Serialize, Clone, Debug)]
pub enum TelegramParseMode {
    MarkdownV2,

    #[serde(rename = "HTML")]
    Html,
}

pub trait TelegramRequest {
    fn method(&self) -> &'static str;
}

/// <https://core.telegram.org/bots/api#sendmessage>
#[derive(Serialize, Debug, Clone)]
pub struct TelegramSendMessage {
    pub chat_id: TelegramChatId,
    pub text: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<TelegramParseMode>,
}

impl TelegramRequest for TelegramSendMessage {
    fn method(&self) -> &'static str {
        "sendMessage"
    }
}

impl TelegramSendMessage {
    pub fn new<S: Into<String>>(chat_id: TelegramChatId, text: S) -> Self {
        Self {
            chat_id,
            text: text.into(),
            parse_mode: None,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct TelegramGetUpdates {
    #[serde(skip_serializing_if = "Option::is_none")]
    offset: Option<i64>,

    timeout: u64,

    allowed_updates: &'static [&'static str],
}

impl TelegramRequest for TelegramGetUpdates {
    fn method(&self) -> &'static str {
        "getUpdates"
    }
}
