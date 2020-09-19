//! Telegram methods for Rhai.

use rhai::{Dynamic, Engine, Map, RegisterFn};

use crate::prelude::*;
use crate::services::telegram::*;

pub fn register_functions(engine: &mut Engine) {
    engine.register_fn("send_message", send_message);
}

// TODO: return `JoinHandle<Result<TelegramMessage>>` to provide a result to a user code.
fn send_message(this: &mut Telegram, unique_id: i64, text: String, options: Map) {
    // FIXME: accept `options` by `&mut`.
    let mut options = options;
    // FIXME: avoid the `clone()`.
    let this = this.clone();
    task::spawn::<_, Result<TelegramMessage>>(async move {
        this.send_message(
            TelegramChatId::UniqueId(unique_id),
            text,
            get_option(&mut options, "parse_mode"),
        )
        .await
    });
}

/// Extract the option from the map and safely cast to the specified type.
fn get_option<T: Send + Clone + Sync + 'static>(options: &mut Map, key: &str) -> Option<T> {
    options.remove(key).and_then(Dynamic::try_cast)
}
