//! [Telegram bot](https://core.telegram.org/bots/api) service able to receive and send messages.

use crate::prelude::*;
use crate::supervisor;
use log::debug;
use reqwest::blocking::Client;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt::Debug;
use std::time::Duration;

// FIXME: set the timeout for `get_updates` individually.
const CLIENT_TIMEOUT_SECS: u64 = 60;

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
    pub fn spawn<'env>(&'env self, scope: &Scope<'env>, service_id: &'env str, bus: &mut Bus) -> Result<()> {
        let tx = bus.add_tx();
        let client = client_builder()
            .timeout(Duration::from_secs(CLIENT_TIMEOUT_SECS + 1))
            .build()?;

        supervisor::spawn(scope, service_id, tx.clone(), move || -> Result<()> {
            let mut offset: Option<i64> = None;
            loop {
                for update in get_updates(&client, &self.secrets.token, offset)?.iter() {
                    offset = offset.max(Some(update.update_id + 1));
                    self.send_readings(&service_id, &tx, &update)?;
                }
                debug!("{}: next offset: {:?}", &service_id, offset);
            }
        })
    }

    /// Send reading messages from the provided Telegram update.
    fn send_readings(&self, service_id: &str, tx: &Sender<Message>, update: &TelegramUpdate) -> Result<()> {
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
}

/// Call [Telegram Bot API](https://core.telegram.org/bots/api) method.
pub fn call_api<P: Serialize + Debug + ?Sized, R: DeserializeOwned>(
    client: &Client,
    token: &str,
    method: &str,
    parameters: &P,
) -> Result<R> {
    debug!("{}({:?})", &method, parameters);
    // FIXME: https://github.com/eigenein/my-iot-rs/issues/44
    let response = client
        .get(&format!("https://api.telegram.org/bot{}/{}", token, method))
        .json(parameters)
        .send()?
        .json::<TelegramResponse<R>>()?;
    if response.ok {
        Ok(response.result.unwrap())
    } else {
        error!("Telegram error: {:?}", response.description);
        Err(InternalError::new(response.description.unwrap()).into())
    }
}

/// <https://core.telegram.org/bots/api#getupdates>
fn get_updates(client: &Client, token: &str, offset: Option<i64>) -> Result<Vec<TelegramUpdate>> {
    call_api(
        client,
        token,
        "getUpdates",
        &json!({
            "offset": offset,
            "limit": null,
            "timeout": CLIENT_TIMEOUT_SECS,
            "allowed_updates": ["message"],
        }),
    )
}

/// <https://core.telegram.org/bots/api#making-requests>
#[derive(Deserialize)]
pub struct TelegramResponse<T> {
    // TODO: enum instead.
    pub ok: bool,
    pub description: Option<String>,
    pub result: Option<T>,
}

#[derive(Deserialize, Debug)]
pub struct TelegramUpdate {
    pub update_id: i64,
    pub message: Option<TelegramMessage>,
}

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
