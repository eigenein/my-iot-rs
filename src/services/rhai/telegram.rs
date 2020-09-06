//! Telegram methods for Rhai.

use bytes::Bytes;
use rhai::{Dynamic, Engine, Map, RegisterFn};

use crate::prelude::*;
use crate::services::telegram::*;

pub fn register_functions(engine: &mut Engine) {
    engine.register_fn("send_message", Telegram::send_message_rhai);
    engine.register_fn("send_video", Telegram::send_video_rhai);
}

// TODO: return `JoinHandle<Result<TelegramMessage>>` to provide a result to a user code.
impl Telegram {
    fn send_message_rhai(&mut self, unique_id: i64, text: &str, options: Map) {
        let mut options = options;
        task::spawn::<_, Result<TelegramMessage>>(Self::call(
            self.secrets.token.clone(),
            MethodCall::SendMessage {
                chat_id: TelegramChatId::UniqueId(unique_id),
                text: text.into(),
                parse_mode: get_option(&mut options, "parse_mode"),
            },
            None,
        ));
    }

    fn send_video_rhai(&mut self, unique_id: i64, bytes: Arc<Bytes>, options: Map) {
        let mut options = options;
        task::spawn::<_, Result<TelegramMessage>>(Self::call(
            self.secrets.token.clone(),
            MethodCall::new_rhai_send_video(TelegramChatId::UniqueId(unique_id), None, &mut options),
            Some(("video".into(), bytes)),
        ));
    }
}

impl MethodCall {
    fn new_rhai_send_video(chat_id: TelegramChatId, video: Option<String>, options: &mut Map) -> Self {
        MethodCall::SendVideo {
            chat_id,
            video,
            caption: get_option(options, "caption"),
            parse_mode: get_option(options, "parse_mode"),
        }
    }
}

/// Extract the option from the map and safely cast to the specified type.
fn get_option<T: Send + Clone + Sync + 'static>(options: &mut Map, key: &str) -> Option<T> {
    options.remove(key).and_then(Dynamic::try_cast)
}
