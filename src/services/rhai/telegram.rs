use rhai::{Dynamic, Engine, RegisterResultFn, Scope};

use crate::prelude::*;
use crate::services::rhai::FnResult;
use crate::services::telegram::*;
use bytes::Bytes;

pub fn register_types(engine: &mut Engine) {
    engine.register_type::<Bytes>();
    engine.register_type::<Telegram>();
    engine.register_type::<TelegramParseMode>();
    engine.register_type::<TelegramMessage>();
}

pub fn push_constants(scope: &mut Scope) {
    scope.push_constant("parse_mode_html", TelegramParseMode::Html);
    scope.push_constant("parse_mode_markdown_v2", TelegramParseMode::MarkdownV2);
}

pub fn register_functions(engine: &mut Engine) {
    engine.register_result_fn(
        "send_message",
        |this: &mut Telegram, unique_id: i64, text: &str| -> FnResult {
            // TODO: `options` parameter.
            Ok(Dynamic::from(
                this.call::<TelegramMessage>(
                    &TelegramMethodCall::SendMessage {
                        chat_id: TelegramChatId::UniqueId(unique_id),
                        text: text.into(),
                        parse_mode: None, // TODO: from `options`.
                    },
                    None,
                )
                .map_err(to_string)?,
            ))
        },
    );

    engine.register_result_fn(
        "send_video",
        |this: &mut Telegram, unique_id: i64, bytes: Bytes| -> FnResult {
            // TODO: `options` parameter.
            Ok(Dynamic::from(
                this.call::<TelegramMessage>(
                    &TelegramMethodCall::SendVideo {
                        chat_id: TelegramChatId::UniqueId(unique_id),
                        video: None, // TODO: override for `&str`.
                    },
                    Some(("video".into(), bytes)),
                )
                .map_err(to_string)?,
            ))
        },
    );
}
