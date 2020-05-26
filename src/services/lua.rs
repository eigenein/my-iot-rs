//! Executes a Lua script for each message allowing to implement any automation scenarios.

use crate::prelude::*;
use crate::services::lua::prelude::*;
use crate::settings::Service;
use regex::Regex;
use std::collections::HashMap;
use uom::si::f64::*;
use uom::si::*;

mod prelude;
mod telegram;

const MESSAGE_ARG_TYPE: &str = "type";
const MESSAGE_ARG_ROOM_TITLE: &str = "room_title";
const MESSAGE_ARG_SENSOR_TITLE: &str = "sensor_title";
const MESSAGE_ARG_VALUE: &str = "value";
const MESSAGE_ARG_TIMESTAMP_MILLIS: &str = "timestamp_millis";

/// Adds Lua scripting and calls message handler for each incoming message.
#[derive(Deserialize, Debug, Clone)]
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
    pub fn spawn(&self, service_id: &str, bus: &mut Bus, services: &HashMap<String, Service>) -> Result<()> {
        let service_id = service_id.to_string();
        let tx = bus.add_tx();
        let rx = bus.add_rx();
        let settings = self.clone();
        let services = services.clone();

        supervisor::spawn(service_id.clone(), tx.clone(), move || -> Result<()> {
            let lua = rlua::Lua::new();
            lua.context(|context| -> Result<()> {
                init_logging(context, &service_id)?;
                init_functions(context, tx.clone())?;
                init_services(context, &services)?;

                info!("[{}] Loading and executing script…", &service_id);
                context.load(&settings.script).set_name(&service_id)?.exec()?;
                let on_message: LuaValue = context.globals().get("onMessage")?;

                info!("[{}] Listening…", &service_id);
                for message in &rx {
                    if settings.is_match(&service_id, &message) {
                        if let LuaValue::Function(on_message) = &on_message {
                            on_message.call::<_, ()>(create_args_table(context, &message)?)?;
                        } else {
                            warn!("[{}] `onMessage` is not defined or not a function", &service_id);
                        }
                    }
                }

                unreachable!()
            })
        })?;

        Ok(())
    }

    /// Checks whether the message matches the filters.
    fn is_match(&self, service_id: &str, message: &Message) -> bool {
        if let Some(ref regex) = self.filter_sensor_ids {
            if !regex.is_match(&message.sensor.id) {
                debug!("[{}] `{}` does not match the filter", &service_id, &message.sensor.id);
                return false;
            }
        }
        if let Some(ref regex) = self.skip_sensor_ids {
            if regex.is_match(&message.sensor.id) {
                debug!("[{}] `{}` is skipped", &service_id, &message.sensor.id);
                return false;
            }
        }
        true
    }
}

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
fn init_logging(context: LuaContext, service_id: &str) -> Result<()> {
    let globals = context.globals();
    globals.set("debug", create_log_function(context, service_id, LogLevel::Debug)?)?;
    globals.set("info", create_log_function(context, service_id, LogLevel::Info)?)?;
    globals.set("warn", create_log_function(context, service_id, LogLevel::Warn)?)?;
    globals.set("error", create_log_function(context, service_id, LogLevel::Error)?)?;
    Ok(())
}

fn create_log_function<S: Into<String>>(context: LuaContext, service_id: S, level: LogLevel) -> LuaResult<LuaFunction> {
    let service_id = service_id.into();
    context.create_function(move |_, message: String| {
        log!(level, "[{}] {}", service_id, message);
        Ok(())
    })
}

/// Provides the custom functions to user code.
fn init_functions(context: LuaContext, tx: Sender<Message>) -> Result<()> {
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
                message.reading.value = ThermodynamicTemperature::new::<thermodynamic_temperature::degree_celsius>(
                    f64::from_lua(value, context)?,
                )
                .into();
            }
            "kelvin" => {
                message.reading.value =
                    ThermodynamicTemperature::new::<thermodynamic_temperature::kelvin>(f64::from_lua(value, context)?)
                        .into();
            }
            "meters" | "metres" => {
                message.reading.value = Length::new::<length::meter>(f64::from_lua(value, context)?).into();
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
            Service::Telegram(telegram) => globals.set(service_id, telegram::Telegram::new(&telegram.token)?),
            _ => Ok(()),
        }?;
    }
    Ok(())
}

impl<'lua> ToLua<'lua> for Value {
    fn to_lua(self, context: LuaContext<'lua>) -> LuaResult<LuaValue> {
        match self {
            Value::Bft(value) => value.to_lua(context),
            Value::Boolean(value) => value.to_lua(context),
            Value::Counter(value) | Value::DataSize(value) => value.to_lua(context),
            Value::Duration(value) => value.value.to_lua(context),
            Value::ImageUrl(value) | Value::Text(value) => value.to_lua(context),
            Value::Length(value) => value.value.to_lua(context),
            Value::None => Ok(LuaValue::Nil),
            Value::Rh(value) => value.to_lua(context),
            Value::Temperature(value) => value.value.to_lua(context),
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
