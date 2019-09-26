//! [Telegram bot](https://core.telegram.org/bots/api) service able to receive and send messages.

use crate::consts::USER_AGENT;
use crate::message::{Message, Reading, Type};
use crate::value::Value;
use crate::{threading, Result};
use bus::Bus;
use chrono::{DateTime, Utc};
use crossbeam_channel::Sender;
use failure::format_err;
use log::{debug, info};
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::thread;
use std::time::Duration;

const TIMEOUT: u32 = 60;

#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    token: String,
}

/// <https://core.telegram.org/bots/api#getupdates>
#[derive(Serialize)]
struct UpdateParameters {
    offset: Option<i64>,
    limit: Option<u32>,
    timeout: Option<u32>,
    allowed_updates: Option<Vec<String>>,
}

impl Default for UpdateParameters {
    fn default() -> UpdateParameters {
        UpdateParameters {
            offset: None,
            limit: None,
            timeout: Some(TIMEOUT),
            allowed_updates: Some(vec!["message".into()]),
        }
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

pub fn spawn(service_id: &str, settings: &Settings, tx: &Sender<Message>, bus: &mut Bus<Message>) -> Result<()> {
    spawn_producer(service_id, &settings.token, tx)?;
    spawn_consumer(service_id, bus)?;
    Ok(())
}

/// Create a new HTTP client.
fn new_client() -> Result<Client> {
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_static(USER_AGENT));

    Ok(reqwest::Client::builder()
        .gzip(true)
        .timeout(Duration::from_secs((TIMEOUT + 1).into()))
        .default_headers(headers)
        .build()?)
}

/// Spawn thread that listens for Telegram updates and produces reading messages.
fn spawn_producer(service_id: &str, token: &str, tx: &Sender<Message>) -> Result<()> {
    let tx = tx.clone();
    let service_id = service_id.to_string();
    let token = token.to_string();
    let client = new_client()?;

    threading::spawn(format!("my-iot::telegram::producer::{}", service_id), move || {
        let mut offset: Option<i64> = None;
        loop {
            debug!("{}: get updates with offset {:?}", &service_id, offset);
            match get_updates(&client, &token, offset) {
                Ok(ref updates) => {
                    for update in updates.iter() {
                        offset = offset.max(Some(update.update_id + 1));
                        send_readings(&update, &service_id, &tx).unwrap();
                    }
                    debug!("{}: next offset: {:?}", service_id, offset);
                }
                Err(error) => {
                    log::error!("Telegram has failed: {}", error);
                    thread::sleep(Duration::from_millis(60000)); // FIXME: something smarter.
                }
            }
        }
    })?;

    Ok(())
}

/// Spawn thread that listens for `Control` messages and communicates back to Telegram.
fn spawn_consumer(service_id: &str, bus: &mut Bus<Message>) -> Result<()> {
    let rx = bus.add_rx();
    let message_regex = Regex::new(&format!(r"^{}::(?P<chat_id>\-?\d+)::message$", service_id))?;

    threading::spawn(format!("my-iot::telegram::consumer::{}", service_id), move || {
        for message in rx {
            if message.type_ != Type::Control {
                continue;
            }
            if let Some(captures) = message_regex.captures(&message.reading.sensor) {
                info!(
                    "Caught message to chat #{}: {:?}",
                    &captures.get(1).unwrap().as_str(),
                    &message.reading.value
                ); // TODO
            }
        }
        unreachable!();
    })?;

    Ok(())
}

/// <https://core.telegram.org/bots/api#getupdates>
fn get_updates(client: &Client, token: &str, offset: Option<i64>) -> Result<Vec<TelegramUpdate>> {
    // TODO: make generic `call` method.
    client
        .get(&format!("https://api.telegram.org/bot{}/getUpdates", token))
        .json(&UpdateParameters {
            offset,
            ..Default::default()
        })
        .send()?
        .json::<TelegramResponse<Vec<TelegramUpdate>>>()
        .map_err(Into::into)
        .and_then(|response| {
            if response.ok {
                Ok(response.result.unwrap())
            } else {
                Err(format_err!("{}", response.description.unwrap()))
            }
        })
}

/// Send reading messages from the provided Telegram update.
fn send_readings(update: &TelegramUpdate, service_id: &str, tx: &Sender<Message>) -> Result<()> {
    debug!("{}: {:?}", service_id, &update);

    if let Some(ref message) = update.message {
        if let Some(ref text) = message.text {
            tx.send(Message {
                type_: Type::OneOff,
                reading: Reading {
                    sensor: format!("{}::{}::message", service_id, message.chat.id),
                    value: Value::Text(text.into()),
                    timestamp: message.date.into(),
                },
            })?;
        }
    }

    Ok(())
}
