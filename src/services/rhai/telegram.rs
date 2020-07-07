use rhai::{Engine, RegisterResultFn, Scope};

use crate::prelude::*;
use crate::services::rhai::FnResult;
use crate::services::telegram::*;

pub fn register_types(engine: &mut Engine) {
    engine.register_type::<Telegram>();
    engine.register_type::<TelegramSendMessage>();
    engine.register_type::<TelegramParseMode>();
}

pub fn push_constants(scope: &mut Scope) {
    scope.push_constant("parse_mode_html", TelegramParseMode::Html);
    scope.push_constant("parse_mode_markdown_v2", TelegramParseMode::MarkdownV2);
}

pub fn register_functions(engine: &mut Engine) {
    engine.register_result_fn(
        "send_message",
        |this: &mut Telegram, unique_id: i64, text: &str| -> FnResult {
            Ok(this
                .call::<_, TelegramMessage>(&TelegramSendMessage::new(TelegramChatId::UniqueId(unique_id), text))
                .map_err(to_string)?
                .message_id
                .into())
        },
    );

    engine.register_result_fn(
        "send_video",
        |_this: &mut Telegram, _unique_id: i64, value: Value| -> FnResult {
            match value {
                Value::Video(_content_type, _content) => unimplemented!(),
                _ => Err(format!("{:?} cannot be made into a video", value).into()),
            }
        },
    );
}
