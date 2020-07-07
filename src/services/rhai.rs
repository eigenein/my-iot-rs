use std::process::Command;

use itertools::Itertools;
use rhai::{Array, Dynamic, Engine, EvalAltResult, ImmutableString, RegisterFn, RegisterResultFn, Scope, AST};

use crate::prelude::*;
use crate::settings::Service;

type FnResult = Result<Dynamic, Box<EvalAltResult>>;

mod telegram;

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Rhai {
    script: String,
}

impl Rhai {
    pub fn spawn(self, service_id: String, bus: &mut Bus, services: HashMap<String, Service>) -> Result {
        let tx = bus.add_tx();
        let rx = bus.add_rx();

        thread::Builder::new()
            .name(service_id.clone())
            .spawn(move || -> Result<(), ()> {
                let mut engine = Engine::new();
                engine.set_max_expr_depths(128, 32);
                let ast = self.compile_script(&service_id, &engine)?;
                let mut scope = Scope::new();

                Self::register_types(&mut engine);
                Self::register_global_functions(&service_id, &mut engine);
                Self::register_functions(&mut engine, &tx);
                Self::push_constants(&mut scope);
                Self::push_services(&mut scope, &services);

                let engine = engine;
                self.consume_ast(&service_id, &engine, &ast, &mut scope)?;

                for message in &rx {
                    if let Err(error) = engine.call_fn::<_, Dynamic>(&mut scope, &ast, "on_message", (message,)) {
                        error!("[{}] `on_message` has failed: {}", &service_id, error.to_string());
                    }
                }

                unreachable!();
            })?;

        Ok(())
    }

    /// Safely compiles the script and logs any errors.
    fn compile_script(&self, service_id: &str, engine: &Engine) -> Result<AST, ()> {
        engine
            .compile(&self.script)
            .map_err(|error| error!("[{}] Compilation error: {}", service_id, error.to_string()))
    }

    /// Safely executes the AST and logs any errors.
    fn consume_ast(&self, service_id: &str, engine: &Engine, ast: &AST, scope: &mut Scope) -> Result<(), ()> {
        engine
            .consume_ast_with_scope(scope, &ast)
            .map_err(|error| error!("[{}] Execution error: {}", service_id, error.to_string()))
    }

    fn register_types(engine: &mut Engine) {
        engine.register_type::<MessageType>();
        engine.register_type::<Message>();
        engine.register_type::<Value>();

        telegram::register_types(engine);
    }

    fn push_constants(scope: &mut Scope) {
        scope.push_constant("message_read_non_logged", MessageType::ReadNonLogged);
        scope.push_constant("message_read_logged", MessageType::ReadLogged);
        scope.push_constant("message_write", MessageType::Write);

        telegram::push_constants(scope);
    }

    /// Registers `MessageType` functions.
    fn register_message_type_functions(engine: &mut Engine) {
        engine.register_fn("to_string", to_debug_string::<MessageType>);
        engine.register_fn("print", to_debug_string::<MessageType>);
        engine.register_fn("debug", to_debug_string::<MessageType>);
    }

    /// Registers `Message` functions.
    fn register_message_functions(engine: &mut Engine, tx: &Sender) {
        engine.register_fn("new_message", Message::new::<String>);
        {
            let tx = tx.clone();
            engine.register_fn("send", move |this: &mut Message| {
                this.clone().send_and_forget(&tx);
            });
        }

        engine.register_fn("to_string", to_debug_string::<Message>);
        engine.register_fn("print", to_debug_string::<Message>);
        engine.register_fn("debug", to_debug_string::<Message>);

        engine.register_get_set(
            "sensor_id",
            |this: &mut Message| this.sensor.id.clone(),
            |this: &mut Message, sensor_id: String| {
                this.sensor.id = sensor_id;
            },
        );
        engine.register_get_set(
            "type",
            |this: &mut Message| this.type_,
            |this: &mut Message, type_: MessageType| {
                this.type_ = type_;
            },
        );
        engine.register_get_set(
            "value",
            |this: &mut Message| this.reading.value.clone(),
            |this: &mut Message, value: Value| {
                this.reading.value = value;
            },
        );
        engine.register_get_set(
            "location",
            |this: &mut Message| this.sensor.location.clone(),
            |this: &mut Message, location: String| {
                this.sensor.location = location;
            },
        );
        // TODO: the rest.
    }

    fn register_value_functions(engine: &mut Engine) {
        engine.register_fn("to_string", to_debug_string::<Value>);
        engine.register_fn("print", to_debug_string::<Value>);
        engine.register_get("inner", |this: &mut Value| -> Dynamic {
            match this {
                Value::None => ().into(),
                Value::Temperature(value)
                | Value::Cloudiness(value)
                | Value::Duration(value)
                | Value::Energy(value)
                | Value::Length(value)
                | Value::Power(value)
                | Value::Rh(value)
                | Value::Speed(value)
                | Value::Volume(value)
                | Value::RelativeIntensity(value)
                | Value::BatteryLife(value) => Dynamic::from(*value),
                Value::Counter(value) | Value::DataSize(value) => Dynamic::from(*value),
                Value::ImageUrl(value) | Value::Text(value) => Dynamic::from(value.clone()),
                Value::Boolean(value) => Dynamic::from(*value),
                Value::WindDirection(value) => Dynamic::from(*value),
                Value::Bft(value) => Dynamic::from(*value),
                Value::Video(content_type, content) => {
                    let mut map = rhai::Map::new();
                    map.insert("type".into(), Dynamic::from(content_type.clone()));
                    map.insert("content".into(), Dynamic::from(content.clone()));
                    map.into()
                }
            }
        });
    }

    fn register_global_functions(service_id: &str, engine: &mut Engine) {
        Self::register_logging_functions(service_id, engine);
        Self::register_standard_functions(engine);
    }

    fn register_functions(engine: &mut Engine, tx: &Sender) {
        Self::register_message_type_functions(engine);
        Self::register_message_functions(engine, &tx);
        Self::register_value_functions(engine);

        telegram::register_functions(engine);
    }

    fn register_logging_functions(service_id: &str, engine: &mut Engine) {
        {
            let service_id = service_id.to_string();
            engine.register_fn("error", move |string: &str| error!("[{}] {}", service_id, string));
        }
        {
            let service_id = service_id.to_string();
            engine.register_fn("warning", move |string: &str| warn!("[{}] {}", service_id, string));
        }
        {
            let service_id = service_id.to_string();
            engine.on_print(move |string| info!("[{}] {}", service_id, string));
        }
        {
            let service_id = service_id.to_string();
            engine.on_debug(move |string| debug!("[{}] {}", service_id, string));
        }
    }

    fn register_standard_functions(engine: &mut Engine) {
        engine.register_result_fn("spawn_process", spawn_process);
        engine.register_fn("starts_with", |this: &mut ImmutableString, other: &str| {
            this.starts_with(other)
        });
    }

    /// Assigns the service instances to the inner variables.
    fn push_services<'a>(scope: &mut Scope<'a>, services: &'a HashMap<String, Service>) {
        for (service_id, service) in services.iter() {
            #[allow(clippy::single_match)]
            match service.clone() {
                Service::Telegram(telegram) => scope.push_constant(service_id, telegram),
                _ => (),
            }
        }
    }
}

/// Used to spawn an external process.
fn spawn_process(program: &str, args: Array) -> FnResult {
    debug!(
        "Spawning: {} {:?}",
        program,
        args.iter().map(|arg| arg.to_string()).join(" ")
    );
    Command::new(program)
        .args(args.iter().map(|arg| arg.to_string()))
        .spawn()
        .map_err(to_string)?;
    Ok(().into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_process_ok() -> Result {
        let mut engine = Engine::new();
        Rhai::register_standard_functions(&mut engine);
        engine.eval(r#"spawn_process("echo", ["-n"])"#)?;
        Ok(())
    }
}
