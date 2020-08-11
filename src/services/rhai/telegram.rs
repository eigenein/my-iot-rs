//! Telegram methods for Rhai.

use bytes::Bytes;
use rhai::{Dynamic, Engine, Map, RegisterResultFn};

use crate::prelude::*;
use crate::services::rhai::FnResult;
use crate::services::telegram::method_call::MethodCall;
use crate::services::telegram::*;

pub fn register_functions(engine: &mut Engine) {
    engine.register_result_fn(
        "send_message",
        |this: &mut Telegram, unique_id: i64, text: &str, options: Map| -> FnResult {
            let mut options = options;
            into_fn_result(task::block_on(this.call::<TelegramMessage>(
                &MethodCall::SendMessage {
                    chat_id: TelegramChatId::UniqueId(unique_id),
                    text: text.into(),
                    parse_mode: get_option(&mut options, "parse_mode"),
                },
                None,
            )))
        },
    );

    engine.register_result_fn(
        "send_video",
        |this: &mut Telegram, unique_id: i64, bytes: Arc<Bytes>, options: Map| -> FnResult {
            let mut options = options;
            into_fn_result(task::block_on(this.call::<TelegramMessage>(
                &MethodCall::new_send_video(TelegramChatId::UniqueId(unique_id), None, &mut options),
                Some(("video".into(), bytes)),
            )))
        },
    );
}

impl MethodCall {
    fn new_send_video(chat_id: TelegramChatId, video: Option<String>, options: &mut Map) -> Self {
        MethodCall::SendVideo {
            chat_id,
            video,
            caption: get_option(options, "caption"),
            parse_mode: get_option(options, "parse_mode"),
        }
    }
}

/// Converts an application `Result<T>` into Rhai function result.
fn into_fn_result<T: Sync + Send + Clone + 'static>(result: Result<T>) -> FnResult {
    result
        .map(Dynamic::from)
        .map_err(|error| error.to_string())
        .map_err(Into::into)
}

/// Extract the option from the map and safely cast to the specified type.
fn get_option<T: Send + Clone + Sync + 'static>(options: &mut Map, key: &str) -> Option<T> {
    options.remove(key).and_then(Dynamic::try_cast)
}
