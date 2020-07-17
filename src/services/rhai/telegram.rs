use rhai::{Dynamic, Engine, Map, RegisterResultFn};

use crate::prelude::*;
use crate::services::rhai::FnResult;
use crate::services::telegram::*;
use bytes::Bytes;

pub fn register_functions(engine: &mut Engine) {
    engine.register_result_fn(
        "send_message",
        |this: &mut Telegram, unique_id: i64, text: &str, options: Map| -> FnResult {
            Ok(Dynamic::from(
                this.call::<TelegramMessage>(
                    &TelegramMethodCall::SendMessage {
                        chat_id: TelegramChatId::UniqueId(unique_id),
                        text: text.into(),
                        parse_mode: get_option(&options, "parse_mode"),
                    },
                    None,
                )
                .map_err(to_string)?,
            ))
        },
    );

    engine.register_result_fn(
        "send_video",
        |this: &mut Telegram, unique_id: i64, bytes: Arc<Bytes>, options: Map| -> FnResult {
            // TODO: `options` parameter.
            Ok(Dynamic::from(
                this.call::<TelegramMessage>(
                    &TelegramMethodCall::SendVideo {
                        chat_id: TelegramChatId::UniqueId(unique_id),
                        video: None, // TODO: override for `&str`.
                        caption: get_option(&options, "caption"),
                        parse_mode: get_option(&options, "parse_mode"),
                    },
                    Some(("video".into(), bytes)),
                )
                .map_err(to_string)?,
            ))
        },
    );
}

/// Extract the option from the map and safely cast to the specified type.
fn get_option<T: Send + Clone + Sync + 'static>(options: &Map, key: &str) -> Option<T> {
    options.get(key).cloned().and_then(Dynamic::try_cast)
}
