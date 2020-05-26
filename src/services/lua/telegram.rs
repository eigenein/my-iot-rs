//! Telegram service for Lua.

use crate::prelude::*;
use crate::services::lua::prelude::*;
use crate::services::telegram::{call_api, new_client, TelegramMessage};
use reqwest::blocking::Client;
use rlua::prelude::*;
use serde_json::{json, Value as JsonValue};

pub struct Telegram {
    token: String,
    client: Client,
}

impl Telegram {
    pub fn new<T: Into<String>>(token: T) -> Result<Self> {
        Ok(Self {
            token: token.into(),
            client: new_client()?,
        })
    }
}

enum TelegramChatId {
    UniqueId(i64),
    Username(String),
}

impl<'lua> FromLua<'lua> for TelegramChatId {
    fn from_lua(lua_value: LuaValue<'lua>, _: LuaContext<'lua>) -> LuaResult<Self> {
        match lua_value {
            LuaValue::String(chat_id) => Ok(TelegramChatId::Username(chat_id.to_str()?.into())),
            LuaValue::Integer(chat_id) => Ok(TelegramChatId::UniqueId(chat_id)),
            _ => Err(rlua::Error::RuntimeError(format!(
                "`{:?}` cannot be made into a chat ID, it must be a string or an integer",
                &lua_value,
            ))),
        }
    }
}

impl From<TelegramChatId> for JsonValue {
    fn from(chat_id: TelegramChatId) -> Self {
        match chat_id {
            TelegramChatId::UniqueId(unique_id) => JsonValue::Number(unique_id.into()),
            TelegramChatId::Username(username) => JsonValue::String(username),
        }
    }
}

impl UserData for Telegram {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("sendMessage", |_, self_, (chat_id, text): (TelegramChatId, String)| {
            call_api::<_, TelegramMessage>(
                &self_.client,
                &self_.token,
                "sendMessage",
                &json!({
                    "chat_id": Into::<JsonValue>::into(chat_id),
                    "text": &text,
                }),
            )
            .map_err(|err| LuaError::RuntimeError(err.to_string()))?;
            Ok(())
        });
    }
}
