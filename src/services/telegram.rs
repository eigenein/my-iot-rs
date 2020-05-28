//! [Telegram bot](https://core.telegram.org/bots/api) service able to receive and send messages.

use crate::consts::USER_AGENT;
use crate::prelude::*;
use crate::supervisor;
use chrono::{DateTime, Utc};
use log::debug;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt::Debug;
use std::time::Duration;

const CLIENT_TIMEOUT_SECS: u64 = 60;
const CLIENT_TIMEOUT: Duration = Duration::from_secs(CLIENT_TIMEOUT_SECS);

#[derive(Deserialize, Debug, Clone)]
pub struct Telegram {
    pub token: String,
}

impl Telegram {
    pub fn spawn<'env>(&'env self, scope: &Scope<'env>, service_id: &'env str, bus: &mut Bus) -> Result<()> {
        let tx = bus.add_tx();
        let client = new_client()?;

        supervisor::spawn(scope, service_id, tx.clone(), move || -> Result<()> {
            let mut offset: Option<i64> = None;
            loop {
                for update in get_updates(&client, &self.token, offset)?.iter() {
                    offset = offset.max(Some(update.update_id + 1));
                    send_readings(&service_id, &tx, &update)?;
                }
                debug!("{}: next offset: {:?}", &service_id, offset);
            }
        })?;

        Ok(())
    }
}

pub fn new_client() -> Result<Client> {
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_static(USER_AGENT));

    Ok(Client::builder()
        .gzip(true)
        .timeout(CLIENT_TIMEOUT)
        .default_headers(headers)
        .build()?)
}

/// Send reading messages from the provided Telegram update.
fn send_readings(service_id: &str, tx: &Sender<Message>, update: &TelegramUpdate) -> Result<()> {
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

/// Call [Telegram Bot API](https://core.telegram.org/bots/api) method.
pub fn call_api<P: Serialize + Debug + ?Sized, R: DeserializeOwned>(
    client: &Client,
    token: &str,
    method: &str,
    parameters: &P,
) -> Result<R> {
    debug!("{}({:?})", &method, parameters);
    // FIXME: https://github.com/eigenein/my-iot-rs/issues/44
    client
        .get(&format!("https://api.telegram.org/bot{}/{}", token, method))
        .json(parameters)
        .send()?
        .json::<TelegramResponse<R>>()
        .map_err(Into::into)
        .and_then(|response| {
            if response.ok {
                Ok(response.result.unwrap())
            } else {
                error!("Telegram error: {:?}", response.description);
                Err(InternalError::new(response.description.unwrap()).into())
            }
        })
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
struct TelegramResponse<T> {
    ok: bool,
    description: Option<String>,
    result: Option<T>,
}

#[derive(Deserialize, Debug)]
struct TelegramUpdate {
    update_id: i64,
    message: Option<TelegramMessage>,
}

/// <https://core.telegram.org/bots/api#message>
#[derive(Deserialize, Debug)]
pub struct TelegramMessage {
    message_id: i64,

    #[serde(deserialize_with = "chrono::serde::ts_seconds::deserialize")]
    date: DateTime<Utc>,

    chat: TelegramChat,
    text: Option<String>,
}

/// <https://core.telegram.org/bots/api#chat>
#[derive(Deserialize, Debug)]
pub struct TelegramChat {
    id: i64,
}
