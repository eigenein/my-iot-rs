//! Executes a Lua script for each message allowing to implement any automation scenarios.

use crate::prelude::*;
use crate::services::lua::prelude::*;
use regex::Regex;
use uom::si::f64::*;
use uom::si::*;

mod prelude;

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

impl Service for Lua {
    fn spawn(&self, service_id: &str, _db: &Arc<Mutex<Connection>>, bus: &mut Bus) -> Result<()> {
        let service_id = service_id.to_string();
        let tx = bus.add_tx();
        let rx = bus.add_rx();
        let settings = self.clone();

        supervisor::spawn(service_id.clone(), tx.clone(), move || -> Result<()> {
            let lua = rlua::Lua::new();
            lua.context(|context| -> Result<()> {
                init_globals(context, &service_id, tx.clone())?;

                info!("[{}] Loading and executing script…", &service_id);
                context.load(&settings.script).set_name(&service_id)?.exec()?;

                let globals = context.globals();
                let on_message: LuaValue = globals.get("onMessage")?;

                info!("[{}] Listening…", &service_id);
                for message in &rx {
                    if let Some(ref regex) = settings.filter_sensor_ids {
                        if !regex.is_match(&message.sensor.sensor_id) {
                            debug!(
                                "[{}] `{}` does not match the filter",
                                &service_id, &message.sensor.sensor_id
                            );
                            continue;
                        }
                    }
                    if let Some(ref regex) = settings.skip_sensor_ids {
                        if regex.is_match(&message.sensor.sensor_id) {
                            debug!("[{}] `{}` is skipped", &service_id, &message.sensor.sensor_id);
                            continue;
                        }
                    }
                    if let LuaValue::Function(on_message) = &on_message {
                        on_message.call::<_, ()>(create_args_table(context, &message)?)?;
                    } else {
                        warn!("[{}] `onMessage` is not defined or not a function", &service_id);
                    }
                }

                unreachable!()
            })
        })?;

        Ok(())
    }
}

fn create_args_table<'lua>(context: LuaContext<'lua>, message: &Message) -> LuaResult<LuaTable<'lua>> {
    let args = context.create_table()?;
    args.set("sensor_id", message.sensor.sensor_id.clone())?;
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

fn init_globals(context: LuaContext, service_id: &str, tx: Sender<Message>) -> Result<()> {
    init_logging(context, service_id)?;
    init_functions(context, tx)?;
    Ok(())
}

/// Expose logging functions to the context.
fn init_logging(context: LuaContext, service_id: &str) -> Result<()> {
    let globals = context.globals();

    globals.set(
        "debug",
        create_log_function(context, service_id.into(), LogLevel::Debug)?,
    )?;
    globals.set("info", create_log_function(context, service_id.into(), LogLevel::Info)?)?;
    globals.set("warn", create_log_function(context, service_id.into(), LogLevel::Warn)?)?;
    globals.set(
        "error",
        create_log_function(context, service_id.into(), LogLevel::Error)?,
    )?;

    Ok(())
}

fn create_log_function(context: LuaContext, service_id: String, level: LogLevel) -> LuaResult<LuaFunction> {
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
                let mut composer = Composer::new(sensor_id).type_(type_);
                if let Some(args) = args {
                    composer = build_message(composer, context, args)?;
                }
                composer.message.send_and_forget(&tx);
                Ok(())
            },
        )?,
    )?;
    Ok(())
}

/// Uses `composer` to build a message from the arguments provided by user in `sendMessage` call.
fn build_message<'lua>(mut composer: Composer, context: LuaContext<'lua>, args: LuaTable<'lua>) -> LuaResult<Composer> {
    for item in args.pairs::<String, LuaValue>() {
        let (key, value) = item?;
        match key.as_ref() {
            MESSAGE_ARG_ROOM_TITLE => {
                composer = composer.room_title(String::from_lua(value, context)?);
            }
            MESSAGE_ARG_SENSOR_TITLE => {
                composer = composer.title(String::from_lua(value, context)?);
            }
            MESSAGE_ARG_TIMESTAMP_MILLIS => {
                composer = composer.timestamp(Local.timestamp_millis(i64::from_lua(value, context)?));
            }
            "bft" | "beaufort" => {
                composer = composer.value(Value::Bft(u8::from_lua(value, context)?));
            }
            "counter" => {
                composer = composer.value(Value::Counter(u64::from_lua(value, context)?));
            }
            "image_url" => {
                composer = composer.value(Value::ImageUrl(String::from_lua(value, context)?));
            }
            "bool" | "boolean" => {
                composer = composer.value(Value::Boolean(bool::from_lua(value, context)?));
            }
            "wind_direction" | "wind" => {
                composer = composer.value(Value::WindDirection(PointOfTheCompass::from_lua(value, context)?));
            }
            "data_size" => {
                composer = composer.value(Value::DataSize(u64::from_lua(value, context)?));
            }
            "text" => {
                composer = composer.value(Value::Text(String::from_lua(value, context)?));
            }
            "rh" | "humidity" => {
                composer = composer.value(Value::Rh(f64::from_lua(value, context)?));
            }
            "celsius" => {
                composer = composer.value(
                    ThermodynamicTemperature::new::<thermodynamic_temperature::degree_celsius>(f64::from_lua(
                        value, context,
                    )?),
                );
            }
            "kelvin" => {
                composer = composer.value(ThermodynamicTemperature::new::<thermodynamic_temperature::kelvin>(
                    f64::from_lua(value, context)?,
                ));
            }
            "meters" | "metres" => {
                composer = composer.value(Length::new::<length::meter>(f64::from_lua(value, context)?));
            }
            "enable_notification" => {
                composer.message.metadata.enable_notification = Some(bool::from_lua(value, context)?);
            }
            _ => warn!("{} = {:?} can't be made into an argument", &key, &value),
        }
    }
    Ok(composer)
}

impl<'lua> ToLua<'lua> for Value {
    fn to_lua(self, context: LuaContext<'lua>) -> LuaResult<LuaValue> {
        match self {
            Value::None => Ok(LuaValue::Nil),
            Value::Bft(value) => value.to_lua(context),
            Value::Boolean(value) => value.to_lua(context),
            Value::Counter(value) | Value::DataSize(value) => value.to_lua(context),
            Value::ImageUrl(value) | Value::Text(value) => value.to_lua(context),
            Value::Length(value) => value.value.to_lua(context),
            Value::Rh(value) => value.to_lua(context),
            Value::Temperature(value) => value.value.to_lua(context),
            Value::WindDirection(value) => value.to_lua(context),
            Value::Duration(value) => value.value.to_lua(context),
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
