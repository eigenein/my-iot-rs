//! Telegram service for Lua.

use rlua::prelude::*;

use crate::services::lua::prelude::*;
use crate::services::telegram::{Telegram, TelegramChatId, TelegramMessage, TelegramSendMessage};

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

impl UserData for Telegram {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("sendMessage", |_, this, (chat_id, text): (TelegramChatId, String)| {
            this.call::<_, TelegramMessage>(&TelegramSendMessage::new(chat_id, text))
                .map_err(|err| LuaError::RuntimeError(err.to_string()))?;
            Ok(())
        });
    }
}
