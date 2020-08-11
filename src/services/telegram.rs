//! [Telegram bot](https://core.telegram.org/bots/api) service which is able to receive and send messages.

use std::fmt::Debug;
use std::time::Duration;

use bytes::Bytes;
use log::debug;
use reqwest::multipart::{Form, Part};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use method_call::MethodCall;

use crate::prelude::*;
use crate::services::prelude::*;

pub mod method_call;

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
    pub fn spawn(self, service_id: String, bus: &mut Bus) -> Result {
        let mut tx = bus.add_tx();

        task::spawn(async move {
            let mut offset: Option<i64> = None;
            loop {
                match self.loop_(&service_id, offset, &mut tx).await {
                    Ok(new_offset) => offset = new_offset,
                    Err(error) => {
                        error!("Failed to refresh the sensors: {}", error.to_string());
                        task::sleep(MINUTE).await;
                    }
                }
            }
        });

        Ok(())
    }

    async fn loop_(&self, service_id: &str, offset: Option<i64>, tx: &mut Sender) -> Result<Option<i64>> {
        let mut offset = offset;
        for update in self.get_updates(offset).await?.iter() {
            offset = offset.max(Some(update.update_id + 1));
            self.send_readings(&service_id, tx, &update).await?;
        }
        debug!("{}: next offset: {:?}", &service_id, offset);
        Ok(offset)
    }

    /// Send reading messages from the provided Telegram update.
    async fn send_readings(&self, service_id: &str, tx: &mut Sender, update: &TelegramUpdate) -> Result {
        debug!("{}: {:?}", service_id, &update);

        if let Some(ref message) = update.message {
            if let Some(ref text) = message.text {
                Message::new(format!("{}::{}::message", service_id, message.chat.id))
                    .type_(MessageType::ReadNonLogged)
                    .value(Value::Text(text.into()))
                    .timestamp(message.date)
                    .send_to(tx)
                    .await;
            }
        }

        Ok(())
    }
}

/// API.
impl Telegram {
    /// <https://core.telegram.org/bots/api#getupdates>
    async fn get_updates(&self, offset: Option<i64>) -> Result<Vec<TelegramUpdate>> {
        self.call(
            &MethodCall::GetUpdates {
                offset,
                timeout: GET_UPDATES_TIMEOUT_SECS,
                allowed_updates: &["message"],
            },
            None,
        )
        .await
    }

    /// Calls a [Telegram Bot API](https://core.telegram.org/bots/api) method.
    pub async fn call<R: DeserializeOwned>(
        &self,
        call: &MethodCall,
        input_file: Option<(String, Arc<Bytes>)>,
    ) -> Result<R> {
        debug!("{:?}", call);

        let url = format!("https://api.telegram.org/bot{}/{}", self.secrets.token, call.url_part(),);

        let mut request = match input_file {
            Some((field_name, bytes)) => CLIENT
                .post(&url)
                .query(call)
                .multipart(Form::new().part(field_name, Part::bytes(bytes.to_vec()).file_name(""))),
            None => CLIENT.get(&url).json(call),
        };

        // `GetUpdates` requires a timeout that is at least as long as the one in the request itself.
        if let MethodCall::GetUpdates { .. } = call {
            request = request.timeout(Duration::from_secs(GET_UPDATES_TIMEOUT_SECS + 1));
        }

        match request.send().await?.json::<TelegramResponse<R>>().await? {
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

    #[allow(unused)]
    Username(String),
}

/// <https://core.telegram.org/bots/api#message>
#[derive(Deserialize, Debug, Clone)]
pub struct TelegramMessage {
    pub message_id: i64,

    #[serde(deserialize_with = "chrono::serde::ts_seconds::deserialize")]
    pub date: DateTime<Utc>,

    pub chat: TelegramChat,
    pub text: Option<String>,
}

/// <https://core.telegram.org/bots/api#chat>
#[derive(Deserialize, Debug, Clone)]
pub struct TelegramChat {
    pub id: i64,
}
