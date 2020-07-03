use std::process::Command;

use rhai::{Array, Dynamic, Engine, EvalAltResult, RegisterFn, RegisterResultFn, Scope, AST};

use crate::prelude::*;
use crate::services::telegram::*;
use crate::settings::Service;
use std::sync::Arc;

type FnResult = Box<EvalAltResult>;

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Rhai {
    script: String,
}

impl Rhai {
    pub fn spawn(self, service_id: String, bus: &mut Bus, services: HashMap<String, Service>) -> Result<()> {
        let tx = bus.add_tx();
        let rx = bus.add_rx();

        thread::Builder::new()
            .name(service_id.clone())
            .spawn(move || -> Result<(), ()> {
                let mut engine = Engine::new();
                engine.set_max_expr_depths(128, 32);
                let ast = self.compile_script(&service_id, &engine)?;
                let mut scope = Scope::new();

                Self::register_message_type_functions(&mut engine);
                Self::register_message_functions(&mut engine, &tx);
                Self::register_value_functions(&mut engine);
                Self::push_constants(&mut scope);
                Self::register_global_functions(&mut engine);
                Self::register_service_functions(&mut engine);
                Self::set_service_values(&mut scope, &services);

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

    fn push_constants(scope: &mut Scope) {
        scope.push_constant("message_read_non_logged", MessageType::ReadNonLogged);
        scope.push_constant("message_read_logged", MessageType::ReadLogged);
        scope.push_constant("message_write", MessageType::Write);
    }

    /// Registers `MessageType` functions.
    fn register_message_type_functions(engine: &mut Engine) {
        engine.register_type::<MessageType>();

        engine.register_fn("to_string", to_debug_string::<MessageType>);
        engine.register_fn("print", to_debug_string::<MessageType>);
        engine.register_fn("debug", to_debug_string::<MessageType>);
    }

    /// Registers `Message` functions.
    fn register_message_functions(engine: &mut Engine, tx: &Sender) {
        engine.register_type::<Message>();

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
        // TODO: more getters and setters.
    }

    /// Registers `Value` functions.
    fn register_value_functions(engine: &mut Engine) {
        engine.register_type::<Value>();

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
                | Value::RelativeIntensity(value) => Dynamic::from(*value),
                Value::Counter(value) | Value::DataSize(value) => Dynamic::from(*value),
                Value::ImageUrl(value) | Value::Text(value) => Dynamic::from(value.clone()),
                Value::Boolean(value) => Dynamic::from(*value),
                Value::WindDirection(value) => Dynamic::from(*value),
                Value::Bft(value) => Dynamic::from(*value),
            }
        });
    }

    fn register_global_functions(engine: &mut Engine) {
        engine.on_print(|string| info!("{}", string));
        engine.on_debug(|string| debug!("{}", string));
        engine.register_result_fn("spawn_process", spawn_process);
    }

    /// Assigns the service instances to the inner variables.
    fn set_service_values<'a>(scope: &mut Scope<'a>, services: &'a HashMap<String, Service>) {
        for (service_id, service) in services.iter() {
            match service.clone() {
                Service::Telegram(telegram) => scope.set_value(service_id, Arc::new(telegram)),
                _ => scope.set_value(service_id, ()),
            }
        }
    }

    /// Registers service functions available for a user.
    fn register_service_functions(engine: &mut Engine) {
        Self::register_telegram_functions(engine);
    }

    fn register_telegram_functions(engine: &mut Engine) {
        engine.register_type::<Arc<Telegram>>();
        engine.register_type::<TelegramChatId>();

        engine.register_fn("telegram_unique_id", |unique_id| {
            Dynamic::from(TelegramChatId::UniqueId(unique_id))
        });
        engine.register_fn("telegram_username", |username| {
            Dynamic::from(TelegramChatId::Username(username))
        });

        engine.register_result_fn(
            "send_message",
            |this: &mut Arc<Telegram>, chat_id: TelegramChatId, text: &str| -> Result<Dynamic, FnResult> {
                Ok(this
                    .send_message(&SendMessageRequest {
                        chat_id,
                        text: text.into(),
                    })
                    .map_err(to_string)?
                    .message_id
                    .into())
            },
        );
    }
}

/// Used to spawn an external process.
fn spawn_process(program: &str, args: Array) -> Result<Dynamic, FnResult> {
    Command::new(program)
        .args(args.iter().map(|arg| arg.to_string()))
        .spawn()
        .map_err(to_string)?;
    Ok(().into())
}

#[cfg(test)]
mod tests {
    use crate::Result;

    use super::*;

    #[test]
    fn spawn_process_ok() -> Result<()> {
        let mut engine = Engine::new();
        Rhai::register_global_functions(&mut engine);
        engine.eval(r#"spawn_process("echo", ["-n"])"#)?;
        Ok(())
    }
}
