//! [Telegram bot](https://core.telegram.org/bots/api) service able to receive and send messages.

use crate::consts::USER_AGENT;
use crate::prelude::*;
use crate::supervisor;
use chrono::{DateTime, Utc};
use crossbeam_channel::Sender;
use log::debug;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt::Debug;
use std::time::Duration;

const CLIENT_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Deserialize, Debug)]
pub struct Telegram {
    token: String,
}

impl Telegram {
    pub fn spawn(&self, service_id: &str, bus: &mut Bus) -> Result<()> {
        let service_id = service_id.to_string();
        let tx = bus.add_tx();
        let token = self.token.clone();
        let client = new_client()?;

        supervisor::spawn(service_id.clone(), tx.clone(), move || -> Result<()> {
            let mut offset: Option<i64> = None;
            loop {
                for update in get_updates(&client, &token, offset)?.iter() {
                    offset = offset.max(Some(update.update_id + 1));
                    send_readings(&service_id, &tx, &update)?;
                }
                debug!("{}: next offset: {:?}", &service_id, offset);
            }
        })?;

        Ok(())
    }
}

fn new_client() -> Result<Client> {
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
fn call_api<P: Serialize + Debug + ?Sized, R: DeserializeOwned>(
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
            "timeout": CLIENT_TIMEOUT,
            "allowed_updates": ["message"],
        }),
    )
}

/// <https://core.telegram.org/bots/api#sendmessage>
#[allow(dead_code)]
fn send_message<T: AsRef<str>>(
    client: &Client,
    token: &str,
    chat_id: TelegramChatId,
    text: T,
    disable_notification: bool,
) -> Result<TelegramMessage> {
    call_api(
        client,
        token,
        "sendMessage",
        &json!({
            "chat_id": chat_id,
            "text": text.as_ref(),
            "disable_notification": disable_notification,
        }),
    )
}

/// <https://core.telegram.org/bots/api#sendphoto>
#[allow(dead_code)]
fn send_photo<P>(
    client: &Client,
    token: &str,
    chat_id: TelegramChatId,
    photo: P,
    disable_notification: bool,
    caption: Option<String>,
) -> Result<TelegramMessage>
where
    P: Into<TelegramFile>,
{
    call_api(
        client,
        token,
        "sendPhoto",
        &json!({
            "chat_id": chat_id,
            "photo": photo.into(),
            "disable_notification": disable_notification,
            "caption": caption, // FIXME: null caption
        }),
    )
}

/// <https://core.telegram.org/bots/api#sendanimation>
#[allow(dead_code)]
fn send_animation<A>(
    client: &Client,
    token: &str,
    chat_id: TelegramChatId,
    animation: A,
    disable_notification: bool,
    caption: Option<String>,
) -> Result<TelegramMessage>
where
    A: Into<TelegramFile>,
{
    call_api(
        client,
        token,
        "sendAnimation",
        &json!({
            "chat_id": chat_id,
            "animation": animation.into(),
            "disable_notification": disable_notification,
            "caption": caption, // FIXME: null caption
        }),
    )
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
enum TelegramChatId {
    UniqueId(i64),
    Username(String),
}

impl From<&str> for TelegramChatId {
    fn from(string: &str) -> Self {
        match string.parse::<i64>() {
            Ok(chat_id) => TelegramChatId::UniqueId(chat_id),
            Err(_) => TelegramChatId::Username(string.into()),
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
enum TelegramFile {
    Url(String),
}

impl From<&String> for TelegramFile {
    fn from(string: &String) -> Self {
        TelegramFile::Url(string.into())
    }
}

/// <https://core.telegram.org/bots/api#making-requests>
#[derive(Deserialize, Debug)]
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
struct TelegramMessage {
    message_id: i64,

    #[serde(deserialize_with = "chrono::serde::ts_seconds::deserialize")]
    date: DateTime<Utc>,

    chat: TelegramChat,
    text: Option<String>,
}

/// <https://core.telegram.org/bots/api#chat>
#[derive(Deserialize, Debug)]
struct TelegramChat {
    id: i64,
}
