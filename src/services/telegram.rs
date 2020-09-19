//! [Telegram bot](https://core.telegram.org/bots/api) service which is able to receive and send messages.

use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use serde_json::json;
use surf::Body;

use crate::prelude::*;
use crate::services::prelude::*;

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Telegram {
    pub secrets: Secrets,
}

/// Secrets section.
#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Secrets {
    pub token: String,
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
                        error!("failed to refresh the sensors: {}", error.to_string());
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

/// Telegram bot API.
impl Telegram {
    /// <https://core.telegram.org/bots/api#getupdates>
    async fn get_updates(&self, offset: Option<i64>) -> Result<Vec<TelegramUpdate>> {
        self.call(
            "getUpdates",
            json!({
                "offset": offset.unwrap_or(0),
                "timeout": crate::services::helpers::middleware::REQUEST_TIMEOUT_SECS - 1,
                "allowed_updates": &["message"],
            }),
        )
        .await
    }

    /// <https://core.telegram.org/bots/api#sendmessage>
    pub async fn send_message<T: Into<String>>(
        &self,
        chat_id: TelegramChatId,
        text: T,
        parse_mode: Option<&'static str>,
    ) -> Result<TelegramMessage> {
        #[derive(Serialize)]
        struct Parameters {
            chat_id: TelegramChatId,
            text: String,

            #[serde(skip_serializing_if = "Option::is_none")]
            parse_mode: Option<&'static str>,
        }

        self.call(
            "sendMessage",
            Body::from_json(&Parameters {
                chat_id,
                text: text.into(),
                parse_mode,
            })
            .map_err(anyhow::Error::msg)?,
        )
        .await
    }

    async fn call<R: serde::de::DeserializeOwned>(&self, method_name: &str, body: impl Into<Body>) -> Result<R> {
        let result = CLIENT
            .post(format!(
                "https://api.telegram.org/bot{}/{}",
                self.secrets.token, method_name,
            ))
            .body(body)
            .recv_json::<TelegramResponse<R>>()
            .await
            .map_err(anyhow::Error::msg)?
            .into();
        log_result(&result, || "Telegram bot API error:");
        result
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

/// Converts `TelegramResponse` into a normal `Result`.
impl<T> From<TelegramResponse<T>> for Result<T> {
    fn from(response: TelegramResponse<T>) -> Self {
        match response {
            TelegramResponse::Result { result } => Ok(result),
            TelegramResponse::Error { description } => Err(anyhow!(description)),
        }
    }
}
