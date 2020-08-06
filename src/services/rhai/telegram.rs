//! Telegram methods for Rhai.

use bytes::Bytes;
use rhai::{Dynamic, Engine, Map, RegisterResultFn};

use crate::prelude::*;
use crate::services::rhai::FnResult;
use crate::services::telegram::*;

pub fn register_functions(engine: &mut Engine) {
    engine.register_result_fn(
        "send_message",
        |this: &mut Telegram, unique_id: i64, text: &str, options: Map| -> FnResult {
            let mut options = options;
            this.call::<TelegramMessage>(
                &TelegramMethodCall::SendMessage {
                    chat_id: TelegramChatId::UniqueId(unique_id),
                    text: text.into(),
                    parse_mode: get_option(&mut options, "parse_mode"),
                },
                None,
            )
            .map(Dynamic::from)
            .map_err(|error| error.to_string())
            .map_err(Into::into)
        },
    );

    engine.register_result_fn(
        "send_video",
        |this: &mut Telegram, unique_id: i64, bytes: Arc<Bytes>, options: Map| -> FnResult {
            let mut options = options;
            this.call::<TelegramMessage>(
                &TelegramMethodCall::SendVideo {
                    chat_id: TelegramChatId::UniqueId(unique_id),
                    video: None, // TODO: override for `&str`.
                    caption: get_option(&mut options, "caption"),
                    parse_mode: get_option(&mut options, "parse_mode"),
                },
                Some(("video".into(), bytes)),
            )
            .map(Dynamic::from)
            .map_err(|error| error.to_string())
            .map_err(Into::into)
        },
    );
}

/// Extract the option from the map and safely cast to the specified type.
fn get_option<T: Send + Clone + Sync + 'static>(options: &mut Map, key: &str) -> Option<T> {
    options.remove(key).and_then(Dynamic::try_cast)
}
