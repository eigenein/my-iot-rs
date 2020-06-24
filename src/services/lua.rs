//! Executes a [Lua](https://www.lua.org/) script for each message allowing to implement any automation scenarios.

use crate::prelude::*;
use crate::services::lua::prelude::*;
use crate::settings::Service;
use regex::Regex;
use std::collections::HashMap;

mod prelude;
mod telegram;

const MESSAGE_ARG_TYPE: &str = "type";
const MESSAGE_ARG_ROOM_TITLE: &str = "room_title";
const MESSAGE_ARG_SENSOR_TITLE: &str = "sensor_title";
const MESSAGE_ARG_VALUE: &str = "value";
const MESSAGE_ARG_TIMESTAMP_MILLIS: &str = "timestamp_millis";
const MESSAGE_ARG_EXPIRES_AT_MILLIS: &str = "expires_at_millis";
const MESSAGE_ARG_EXPIRES_IN_MILLIS: &str = "expires_in_millis";

/// Adds Lua scripting and calls message handler for each incoming message.
#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Lua {
    /// Script body.
    script: String,

    /// If set to `Some(regex)`, ensures that message handler gets called only for sensor IDs
    /// that match specified [pattern](https://docs.rs/regex/).
    #[serde(default, with = "serde_regex")]
    filter_sensor_ids: Option<Regex>,

    /// If set to `Some(regex)`, ensures that message handler doesn't get called for sensor IDs
    /// that match specified [pattern](https://docs.rs/regex/).
    #[serde(default, with = "serde_regex")]
    skip_sensor_ids: Option<Regex>,
}

impl Lua {
    pub fn spawn(self, service_id: String, bus: &mut Bus, services: HashMap<String, Service>) -> Result<()> {
        let tx = bus.add_tx();
        let rx = bus.add_rx();

        let lua = rlua::Lua::new();
        lua.context(|context| -> Result<()> {
            init_logging(context)?;
            init_functions(context, tx.clone())?;
            init_services(context, &services)?;

            info!("Loading and executing the script…");
            context.load(&self.script).set_name(&service_id)?.exec()?;

            Ok(())
        })?;

        thread::Builder::new().name(service_id).spawn(move || {
            lua.context(|context| {
                info!("Listening…");
                for message in &rx {
                    if !(self.is_match(&message)) {
                        continue;
                    }
                    if let Err(error) = context
                        .globals()
                        .get::<_, LuaFunction>("onMessage")
                        .and_then(|on_message| {
                            create_args_table(context, &message).and_then(|args| on_message.call::<_, ()>(args))
                        })
                    {
                        error!("Failed to handle the message: {}", error.to_string());
                    }
                }

                unreachable!()
            });
        })?;

        Ok(())
    }

    /// Checks whether the message matches the filters.
    fn is_match(&self, message: &Message) -> bool {
        if let Some(ref regex) = self.filter_sensor_ids {
            if !regex.is_match(&message.sensor.id) {
                debug!("`{}` does not match the filter", &message.sensor.id);
                return false;
            }
        }
        if let Some(ref regex) = self.skip_sensor_ids {
            if regex.is_match(&message.sensor.id) {
                debug!("`{}` is skipped", &message.sensor.id);
                return false;
            }
        }
        true
    }
}

/// Prepares arguments for `onMessage` call.
fn create_args_table<'lua>(context: LuaContext<'lua>, message: &Message) -> LuaResult<LuaTable<'lua>> {
    let args = context.create_table()?;
    args.set("sensor_id", message.sensor.id.clone())?;
    args.set(MESSAGE_ARG_TYPE, message.type_)?;
    args.set(MESSAGE_ARG_ROOM_TITLE, message.sensor.room_title.clone())?;
    args.set(MESSAGE_ARG_SENSOR_TITLE, message.sensor.title.clone())?;
    args.set(MESSAGE_ARG_VALUE, message.reading.value.clone())?;
    args.set(
        MESSAGE_ARG_TIMESTAMP_MILLIS,
        message.reading.timestamp.timestamp_millis(),
    )?;
    Ok(args)
}

/// Expose logging functions to the context.
fn init_logging(context: LuaContext) -> Result<()> {
    let globals = context.globals();
    globals.set("debug", create_log_function(context, LogLevel::Debug)?)?;
    globals.set("info", create_log_function(context, LogLevel::Info)?)?;
    globals.set("warn", create_log_function(context, LogLevel::Warn)?)?;
    globals.set("error", create_log_function(context, LogLevel::Error)?)?;
    Ok(())
}

fn create_log_function(context: LuaContext, level: LogLevel) -> LuaResult<LuaFunction> {
    context.create_function(move |_, message: String| {
        log!(level, "{}", message);
        Ok(())
    })
}

/// Provides the custom functions to user code.
fn init_functions(context: LuaContext, tx: Sender) -> Result<()> {
    let globals = context.globals();
    globals.set(
        "sendMessage",
        context.create_function(
            move |context, (sensor_id, type_, args): (String, MessageType, Option<LuaTable>)| {
                let mut message = Message::new(sensor_id).type_(type_);
                if let Some(args) = args {
                    enrich_message(&mut message, context, args)?;
                }
                message.send_and_forget(&tx);
                Ok(())
            },
        )?,
    )?;
    Ok(())
}

/// Modify the message from the arguments provided by user in `sendMessage` call.
fn enrich_message<'lua>(message: &mut Message, context: LuaContext<'lua>, args: LuaTable<'lua>) -> LuaResult<()> {
    for item in args.pairs::<String, LuaValue>() {
        let (key, value) = item?;
        match key.as_ref() {
            MESSAGE_ARG_ROOM_TITLE => {
                message.sensor.room_title = FromLua::from_lua(value, context)?;
            }
            MESSAGE_ARG_SENSOR_TITLE => {
                message.sensor.title = FromLua::from_lua(value, context)?;
            }
            MESSAGE_ARG_TIMESTAMP_MILLIS => {
                message.reading.timestamp = Local.timestamp_millis(i64::from_lua(value, context)?);
            }
            MESSAGE_ARG_EXPIRES_AT_MILLIS => {
                message.sensor.expires_at = Local.timestamp_millis(i64::from_lua(value, context)?);
            }
            MESSAGE_ARG_EXPIRES_IN_MILLIS => {
                message.sensor.expires_at =
                    Local::now() + chrono::Duration::milliseconds(i64::from_lua(value, context)?);
            }
            "bft" | "beaufort" => {
                message.reading.value = Value::Bft(u8::from_lua(value, context)?);
            }
            "counter" => {
                message.reading.value = Value::Counter(u64::from_lua(value, context)?);
            }
            "image_url" => {
                message.reading.value = Value::ImageUrl(String::from_lua(value, context)?);
            }
            "bool" | "boolean" => {
                message.reading.value = Value::Boolean(bool::from_lua(value, context)?);
            }
            "wind_direction" | "wind" => {
                message.reading.value = Value::WindDirection(PointOfTheCompass::from_lua(value, context)?);
            }
            "data_size" => {
                message.reading.value = Value::DataSize(u64::from_lua(value, context)?);
            }
            "text" => {
                message.reading.value = Value::Text(String::from_lua(value, context)?);
            }
            "rh" | "humidity" => {
                message.reading.value = Value::Rh(f64::from_lua(value, context)?);
            }
            "celsius" => {
                message.reading.value = Value::Temperature(f64::from_lua(value, context)?);
            }
            "meters" | "metres" => {
                message.reading.value = Value::Length(f64::from_lua(value, context)?);
            }
            _ => warn!("{} = {:?} can't be made into an argument", &key, &value),
        }
    }
    Ok(())
}

fn init_services(context: LuaContext, services: &HashMap<String, Service>) -> Result<()> {
    let globals = context.globals();
    for (service_id, service) in services.iter() {
        let service_id = service_id.to_string();
        match service {
            Service::Telegram(telegram) => globals.set(service_id, telegram.clone()),
            _ => Ok(()),
        }?;
    }
    Ok(())
}

impl<'lua> ToLua<'lua> for Value {
    fn to_lua(self, context: LuaContext<'lua>) -> LuaResult<LuaValue> {
        match self {
            Value::None => Ok(LuaValue::Nil),
            Value::Boolean(value) => value.to_lua(context),
            Value::Bft(value) => value.to_lua(context),
            Value::ImageUrl(value) | Value::Text(value) => value.to_lua(context),
            Value::Counter(value) | Value::DataSize(value) => value.to_lua(context),
            Value::Rh(value)
            | Value::RelativeIntensity(value)
            | Value::Duration(value)
            | Value::Length(value)
            | Value::Temperature(value)
            | Value::Energy(value)
            | Value::Power(value)
            | Value::Volume(value) => value.to_lua(context),
            Value::WindDirection(value) => value.to_lua(context),
        }
    }
}

impl<'lua> ToLua<'lua> for PointOfTheCompass {
    fn to_lua(self, context: LuaContext<'lua>) -> LuaResult<LuaValue<'lua>> {
        match self {
            PointOfTheCompass::East => "EAST",
            PointOfTheCompass::EastNortheast => "EAST_NORTH_EAST",
            PointOfTheCompass::EastSoutheast => "EAST_SOUTH_EAST",
            PointOfTheCompass::North => "NORTH",
            PointOfTheCompass::Northeast => "NORTH_EAST",
            PointOfTheCompass::NorthNortheast => "NORTH_NORTH_EAST",
            PointOfTheCompass::NorthNorthwest => "NORTH_NORTH_WEST",
            PointOfTheCompass::Northwest => "NORTH_WEST",
            PointOfTheCompass::South => "SOUTH",
            PointOfTheCompass::Southeast => "SOUTH_EAST",
            PointOfTheCompass::SouthSoutheast => "SOUTH_SOUTH_EAST",
            PointOfTheCompass::SouthSouthwest => "SOUTH_SOUTH_WEST",
            PointOfTheCompass::Southwest => "SOUTH_WEST",
            PointOfTheCompass::West => "WEST",
            PointOfTheCompass::WestNorthwest => "WEST_NORTH_WEST",
            PointOfTheCompass::WestSouthwest => "WEST_SOUTH_WEST",
        }
        .to_lua(context)
    }
}

impl<'lua> ToLua<'lua> for MessageType {
    fn to_lua(self, context: LuaContext<'lua>) -> LuaResult<LuaValue<'lua>> {
        match self {
            MessageType::ReadSnapshot => "READ_SNAPSHOT",
            MessageType::ReadNonLogged => "READ_NON_LOGGED",
            MessageType::ReadLogged => "READ_LOGGED",
            MessageType::Write => "WRITE",
        }
        .to_lua(context)
    }
}

impl<'lua> FromLua<'lua> for MessageType {
    fn from_lua(lua_value: LuaValue<'lua>, _: LuaContext<'lua>) -> LuaResult<Self> {
        match lua_value {
            LuaValue::String(string) => match string.to_str()? {
                "READ_LOGGED" => Ok(MessageType::ReadLogged),
                "READ_NON_LOGGED" => Ok(MessageType::ReadNonLogged),
                "READ_SNAPSHOT" => Ok(MessageType::ReadSnapshot),
                "WRITE" => Ok(MessageType::Write),
                _ => Err(rlua::Error::RuntimeError(format!(
                    "`{:?}` cannot be made into a message type, unknown value",
                    string,
                ))),
            },
            _ => Err(rlua::Error::RuntimeError(format!(
                "`{:?}` cannot be made into a message type, it must be a string",
                &lua_value,
            ))),
        }
    }
}

impl<'lua> FromLua<'lua> for PointOfTheCompass {
    fn from_lua(lua_value: LuaValue<'lua>, _: LuaContext<'lua>) -> LuaResult<Self> {
        match lua_value {
            LuaValue::String(string) => match string.to_str()? {
                "N" | "NORTH" => Ok(PointOfTheCompass::North),
                "W" | "WEST" => Ok(PointOfTheCompass::West),
                "E" | "EAST" => Ok(PointOfTheCompass::East),
                "S" | "SOUTH" => Ok(PointOfTheCompass::South),
                _ => Err(rlua::Error::RuntimeError(format!(
                    "`{:?}` cannot be made into a point of the compass, unknown value",
                    string,
                ))),
            },
            _ => Err(rlua::Error::RuntimeError(format!(
                "`{:?}` cannot be made into a point of the compass, it must be a string",
                &lua_value,
            ))),
        }
    }
}
