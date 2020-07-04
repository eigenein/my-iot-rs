use rhai::{Engine, RegisterFn, RegisterResultFn, Scope};

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
    engine.register_fn("new_send_message", |unique_id: i64, text: &str| {
        TelegramSendMessage::new(TelegramChatId::UniqueId(unique_id), text)
    });
    engine.register_fn("new_send_message", |username: &str, text: &str| {
        TelegramSendMessage::new(TelegramChatId::Username(username.into()), text)
    });

    engine.register_set(
        "parse_mode",
        |this: &mut TelegramSendMessage, parse_mode: TelegramParseMode| {
            this.parse_mode = Some(parse_mode);
        },
    );

    engine.register_result_fn(
        "call",
        |this: &mut Telegram, request: TelegramSendMessage| -> FnResult {
            Ok(this
                .call::<_, TelegramMessage>(&request)
                .map_err(to_string)?
                .message_id
                .into())
        },
    );
}
