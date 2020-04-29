//! [Telegram bot](https://core.telegram.org/bots/api) service able to receive and send messages.

use crate::consts::USER_AGENT;
use crate::prelude::*;
use crate::supervisor;
use chrono::{DateTime, Utc};
use crossbeam_channel::Sender;
use log::debug;
use regex::Regex;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt::Debug;
use std::time::Duration;

/// > Should be positive, short polling should be used for testing purposes only.
const GET_UPDATES_TIMEOUT: u64 = 60;

/// Global client timeout. Based on `GET_UPDATES_TIMEOUT` because `reqwest` does not allow to set
/// individual reqwest timeout.
static CLIENT_TIMEOUT: Duration = Duration::from_secs(GET_UPDATES_TIMEOUT + 5);

#[derive(Deserialize, Debug)]
pub struct Telegram {
    token: String,
}

impl Service for Telegram {
    fn spawn(&self, service_id: &str, _db: &Arc<Mutex<Connection>>, bus: &mut Bus) -> Result<()> {
        spawn_producer(Context::new(service_id, &self.token)?, bus)?;
        spawn_consumer(Context::new(service_id, &self.token)?, bus)?;
        Ok(())
    }
}

struct Context {
    client: Client,
    service_id: String,
    token: String,
}

impl Context {
    fn new(service_id: &str, token: &str) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_static(USER_AGENT));

        Ok(Self {
            client: Client::builder()
                .gzip(true)
                .timeout(CLIENT_TIMEOUT)
                .default_headers(headers)
                .build()?,
            service_id: service_id.into(),
            token: token.into(),
        })
    }
}

/// Spawn thread that listens for Telegram updates and produces reading messages.
fn spawn_producer(context: Context, bus: &Bus) -> Result<()> {
    let tx = bus.add_tx();

    supervisor::spawn(
        &format!("{}::producer", &context.service_id),
        tx.clone(),
        move || -> Result<()> {
            let mut offset: Option<i64> = None;
            loop {
                for update in get_updates(&context, offset)?.iter() {
                    offset = offset.max(Some(update.update_id + 1));
                    send_readings(&context, &tx, &update).unwrap();
                }
                debug!("{}: next offset: {:?}", &context.service_id, offset);
            }
        },
    )?;

    Ok(())
}

/// Send reading messages from the provided Telegram update.
fn send_readings(context: &Context, tx: &Sender<Message>, update: &TelegramUpdate) -> Result<()> {
    debug!("{}: {:?}", &context.service_id, &update);

    if let Some(ref message) = update.message {
        if let Some(ref text) = message.text {
            tx.send(
                Composer::new(format!("{}::{}::message", &context.service_id, message.chat.id))
                    .type_(MessageType::ReadNonLogged)
                    .value(Value::Text(text.into()))
                    .timestamp(message.date)
                    .into(),
            )?;
        }
    }

    Ok(())
}

/// Spawn thread that listens for `Control` messages and communicates back to Telegram.
fn spawn_consumer(context: Context, bus: &mut Bus) -> Result<()> {
    let message_regex = Regex::new(&format!(
        r"^{}::(?P<chat_id>\-?\d+)::(?P<sensor>\w+)",
        &context.service_id,
    ))?;
    let rx = bus.add_rx();

    supervisor::spawn(format!("{}::consumer", &context.service_id), bus.add_tx(), move || {
        for message in &rx {
            if message.type_ != MessageType::Write {
                continue;
            }
            let (chat_id, sensor) = match message_regex.captures(&message.sensor.sensor_id) {
                Some(captures) => (captures.get(1).unwrap().as_str(), captures.get(2).unwrap().as_str()),
                None => continue,
            };
            let chat_id: TelegramChatId = chat_id.into();
            match message.reading.value {
                Value::Text(ref text) if sensor == "message" => {
                    send_message(
                        &context,
                        chat_id,
                        text,
                        !message.metadata.enable_notification.unwrap_or(true),
                    )?;
                }
                Value::ImageUrl(ref url) if sensor == "photo" => {
                    send_photo(
                        &context,
                        chat_id,
                        url,
                        !message.metadata.enable_notification.unwrap_or(true),
                        message.sensor.title.clone(),
                    )?;
                }
                Value::ImageUrl(ref url) if sensor == "animation" => {
                    send_animation(
                        &context,
                        chat_id,
                        url,
                        !message.metadata.enable_notification.unwrap_or(true),
                        message.sensor.title.clone(),
                    )?;
                }
                value => warn!(
                    "sending `{:?}` to `{}` is not implemented",
                    &value, &message.sensor.sensor_id
                ),
            };
        }
        unreachable!();
    })?;

    Ok(())
}

/// Call [Telegram Bot API](https://core.telegram.org/bots/api) method.
fn call_api<P: Serialize + Debug + ?Sized, R: DeserializeOwned>(
    context: &Context,
    method: &str,
    parameters: &P,
) -> Result<R> {
    debug!("{}({:?})", &method, parameters);
    // FIXME: https://github.com/eigenein/my-iot-rs/issues/44
    context
        .client
        .get(&format!("https://api.telegram.org/bot{}/{}", &context.token, method))
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
fn get_updates(context: &Context, offset: Option<i64>) -> Result<Vec<TelegramUpdate>> {
    call_api(
        context,
        "getUpdates",
        &json!({
            "offset": offset,
            "limit": null,
            "timeout": GET_UPDATES_TIMEOUT,
            "allowed_updates": ["message"],
        }),
    )
}

/// <https://core.telegram.org/bots/api#sendmessage>
fn send_message<T: AsRef<str>>(
    context: &Context,
    chat_id: TelegramChatId,
    text: T,
    disable_notification: bool,
) -> Result<TelegramMessage> {
    call_api(
        context,
        "sendMessage",
        &json!({
            "chat_id": chat_id,
            "text": text.as_ref(),
            "disable_notification": disable_notification,
        }),
    )
}

/// <https://core.telegram.org/bots/api#sendphoto>
fn send_photo<P>(
    context: &Context,
    chat_id: TelegramChatId,
    photo: P,
    disable_notification: bool,
    caption: Option<String>,
) -> Result<TelegramMessage>
where
    P: Into<TelegramFile>,
{
    call_api(
        context,
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
fn send_animation<A>(
    context: &Context,
    chat_id: TelegramChatId,
    animation: A,
    disable_notification: bool,
    caption: Option<String>,
) -> Result<TelegramMessage>
where
    A: Into<TelegramFile>,
{
    call_api(
        context,
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
