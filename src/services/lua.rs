//! Executes a Lua script for each message allowing to implement any automation scenarios.

use crate::core::message::Type as MessageType;
use crate::prelude::*;
use rlua::{Context, Lua, MetaMethod, Table, ToLua, UserData, UserDataMethods};

const MESSAGE_ARG_TYPE: &str = "type";
const MESSAGE_ARG_ROOM_TITLE: &str = "room_title";
const MESSAGE_ARG_SENSOR_TITLE: &str = "sensor_title";
const MESSAGE_ARG_VALUE: &str = "value";
const MESSAGE_ARG_TIMESTAMP: &str = "timestamp";

#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    /// Script body.
    script: String,
}

pub fn spawn(service_id: &str, settings: &Settings, bus: &mut Bus) -> Result<()> {
    let service_id = service_id.to_string();
    let tx = bus.add_tx();
    let rx = bus.add_rx();
    let script = settings.script.clone();

    supervisor::spawn(service_id.clone(), tx.clone(), move || -> Result<()> {
        let lua = Lua::new();
        lua.context(|context| -> Result<()> {
            init_globals(context, &service_id, tx.clone())?;

            info!("[{}] Loading and executing script…", &service_id);
            context.load(&script).set_name(&service_id)?.exec()?;

            let globals = context.globals();
            let on_message: rlua::Value = globals.get("onMessage")?;

            info!("[{}] Listening…", &service_id);
            for message in &rx {
                if let rlua::Value::Function(on_message) = &on_message {
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

fn create_args_table<'lua>(context: Context<'lua>, message: &Message) -> rlua::Result<Table<'lua>> {
    let args = context.create_table()?;
    args.set("sensor_id", message.sensor.sensor_id.clone())?;
    args.set(MESSAGE_ARG_TYPE, message.type_)?;
    args.set(MESSAGE_ARG_ROOM_TITLE, message.sensor.room_title.clone())?;
    args.set(MESSAGE_ARG_SENSOR_TITLE, message.sensor.title.clone())?;
    args.set(MESSAGE_ARG_VALUE, message.reading.value.clone())?;
    args.set(MESSAGE_ARG_TIMESTAMP, message.reading.timestamp.timestamp())?;
    Ok(args)
}

fn init_globals(context: Context, service_id: &str, tx: Sender<Message>) -> Result<()> {
    init_logging(context, service_id)?;
    init_constants(context)?;
    init_functions(context, tx)?;
    Ok(())
}

/// Expose logging functions to the context.
fn init_logging(context: Context, service_id: &str) -> Result<()> {
    let globals = context.globals();
    {
        let service_id = service_id.to_string();
        globals.set(
            "debug",
            context.create_function(move |_, string: String| {
                debug!("[{}] {}", service_id, string);
                Ok(())
            })?,
        )?;
    }
    {
        let service_id = service_id.to_string();
        globals.set(
            "info",
            context.create_function(move |_, string: String| {
                info!("[{}] {}", service_id, string);
                Ok(())
            })?,
        )?;
    }
    {
        let service_id = service_id.to_string();
        globals.set(
            "warn",
            context.create_function(move |_, string: String| {
                warn!("[{}] {}", service_id, string);
                Ok(())
            })?,
        )?;
    }
    {
        let service_id = service_id.to_string();
        globals.set(
            "error",
            context.create_function(move |_, string: String| {
                error!("[{}] {}", service_id, string);
                Ok(())
            })?,
        )?;
    }

    Ok(())
}

fn init_constants(context: Context) -> Result<()> {
    let globals = context.globals();
    globals.set("MESSAGE_READ_LOGGED", MessageType::ReadLogged)?;
    globals.set("MESSAGE_READ_NON_LOGGED", MessageType::ReadNonLogged)?;
    globals.set("MESSAGE_READ_SNAPSHOT", MessageType::ReadSnapshot)?;
    globals.set("MESSAGE_WRITE", MessageType::Write)?;
    Ok(())
}

fn init_functions(context: Context, tx: Sender<Message>) -> Result<()> {
    let globals = context.globals();
    globals.set(
        "sendMessage",
        context.create_function(
            move |_, (sensor_id, type_, _args): (String, MessageType, Option<Table>)| {
                tx.send(Composer::new(sensor_id).type_(type_).message)
                    .map_err(|err| rlua::Error::RuntimeError(err.to_string()))?;
                Ok(())
            },
        )?,
    )?;
    Ok(())
}

impl UserData for MessageType {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_function(MetaMethod::Eq, |_, (left, right): (MessageType, MessageType)| {
            Ok(left == right)
        })
    }
}

impl UserData for crate::core::value::PointOfTheCompass {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_function(
            MetaMethod::Eq,
            |_,
             (left, right): (
                crate::core::value::PointOfTheCompass,
                crate::core::value::PointOfTheCompass,
            )| Ok(left == right),
        )
    }
}

impl<'lua> ToLua<'lua> for Value {
    fn to_lua(self, context: Context<'lua>) -> rlua::Result<rlua::Value> {
        Ok(match self {
            Value::Bft(value) => rlua::Value::Integer(value as i64),
            Value::Boolean(value) => rlua::Value::Boolean(value),
            Value::Counter(value) | Value::DataSize(value) => rlua::Value::Integer(value as i64),
            Value::ImageUrl(value) | Value::Text(value) => value.to_lua(context)?,
            Value::Length(value) => rlua::Value::Number(value.value),
            Value::None => rlua::Value::Nil,
            Value::Rh(value) => rlua::Value::Number(value),
            Value::Temperature(value) => rlua::Value::Number(value.value),
            Value::WindDirection(value) => value.to_lua(context)?,
        })
    }
}
