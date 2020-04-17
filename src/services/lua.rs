//! Executes a Lua script for each message allowing to implement any automation scenarios.

use crate::prelude::*;
use rlua::{Context, Lua, ToLua};

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

    supervisor::spawn(service_id.clone(), tx, move || -> Result<()> {
        let lua = Lua::new();
        lua.context(|context| -> Result<()> {
            init_logging(context, &service_id)?;

            info!("[{}] Loading and executing script…", &service_id);
            context.load(&script).set_name(&service_id)?.exec()?;

            let globals = context.globals();
            let on_message: rlua::Value = globals.get("on_message")?;

            info!("[{}] Listening…", &service_id);
            for message in &rx {
                if let rlua::Value::Function(on_message) = &on_message {
                    let args = context.create_table()?;
                    args.set("sensor_id", message.sensor.sensor_id.clone())?;
                    args.set("type", message.type_)?;
                    on_message.call::<_, ()>(args)?;
                } else {
                    warn!("[{}] `on_message` is not defined or not a function", &service_id);
                }
            }

            unreachable!()
        })
    })?;

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

impl ToLua<'_> for crate::core::message::Type {
    fn to_lua(self, context: Context) -> rlua::Result<rlua::Value> {
        format!("{:?}", self).to_lua(context)
    }
}
