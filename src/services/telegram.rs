//! [Telegram bot](https://core.telegram.org/bots/api) service able to receive and send messages.

use crate::consts::USER_AGENT;
use crate::db::Db;
use crate::message::{Message, Reading, Type};
use crate::services::Service;
use crate::value::Value;
use crate::{threading, Result};
use bus::Bus;
use chrono::{DateTime, Utc};
use crossbeam_channel::Sender;
use failure::format_err;
use log::debug;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const TIMEOUT: u32 = 60;

#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    token: String,
}

/// Telegram bot service.
pub struct Telegram {
    service_id: String,
    client: reqwest::Client,

    /// <https://core.telegram.org/bots/api#making-requests>
    url_prefix: String,
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

impl Service for Telegram {
    fn spawn(self: Box<Self>, _db: Arc<Mutex<Db>>, tx: &Sender<Message>, _rx: &mut Bus<Message>) -> Result<()> {
        let tx = tx.clone();

        threading::spawn(format!("my-iot::telegram:{}", &self.service_id), move || {
            let mut offset: Option<i64> = None;
            loop {
                match self.get_updates(offset) {
                    Ok(ref updates) => {
                        for update in updates.iter() {
                            offset = offset.max(Some(update.update_id + 1));
                            self.send(&update, &tx).unwrap();
                        }
                        debug!("{}: next offset: {:?}", &self.service_id, offset);
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
}

impl Telegram {
    pub fn new(service_id: &str, settings: &Settings) -> Result<Telegram> {
        let mut headers = HeaderMap::new();
        headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_static(USER_AGENT));
        Ok(Telegram {
            service_id: service_id.into(),
            url_prefix: format!("https://api.telegram.org/bot{}/", &settings.token),
            client: reqwest::Client::builder()
                .gzip(true)
                .timeout(Duration::from_secs((TIMEOUT + 1).into()))
                .default_headers(headers)
                .build()?,
        })
    }

    /// <https://core.telegram.org/bots/api#getupdates>
    fn get_updates(&self, offset: Option<i64>) -> Result<Vec<TelegramUpdate>> {
        debug!("{}: get updates with offset {:?}", &self.service_id, offset);
        // TODO: make generic `call` method.
        self.client
            .get(&format!("{}{}", self.url_prefix, "getUpdates"))
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

    fn send(&self, update: &TelegramUpdate, tx: &Sender<Message>) -> Result<()> {
        debug!("{}: {:?}", &self.service_id, &update);

        if let Some(ref message) = update.message {
            if let Some(ref text) = message.text {
                tx.send(Message {
                    type_: Type::OneOff,
                    reading: Reading {
                        sensor: format!("{}::{}::message", self.service_id, message.chat.id),
                        value: Value::Text(text.into()),
                        timestamp: message.date.into(),
                    },
                })?;
            }
        }

        Ok(())
    }
}
