use std::process::Command;

use bytes::Bytes;
use itertools::Itertools;
use regex::Regex;
use rhai::{Array, Dynamic, Engine, EvalAltResult, ImmutableString, RegisterFn, RegisterResultFn, Scope};

use crate::prelude::*;
use crate::settings::Service;

type FnResult = StdResult<Dynamic, Box<EvalAltResult>>;

mod telegram;

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Rhai {
    script: String,

    #[serde(with = "serde_regex", default)]
    sensor_pattern: Option<Regex>,
}

impl Rhai {
    pub fn spawn(self, service_id: String, bus: &mut Bus, services: HashMap<String, Service>) -> Result {
        let mut rx = bus.add_rx();

        let mut engine = Engine::new();
        engine.set_max_expr_depths(128, 32);
        let mut ast = engine.compile(&self.script)?;
        let mut scope = Scope::new();

        Self::register_global_functions(&service_id, &mut engine);
        Self::register_functions(&mut engine, bus.add_tx());
        Self::push_constants(&mut scope);
        Self::push_services(&mut scope, services);

        engine.consume_ast_with_scope(&mut scope, &ast)?;

        task::spawn(async move {
            while let Some(message) = rx.next().await {
                if let Some(pattern) = &self.sensor_pattern {
                    if !pattern.is_match(&message.sensor.id) {
                        debug!("[{}] `{}` is filtered out.", service_id, message.sensor.id);
                        continue;
                    }
                }
                let service_id = service_id.clone();
                let (engine_, ast_, scope_) = task::spawn(async move {
                    if let Err(error) = engine.call_fn::<_, Dynamic>(&mut scope, &ast, "on_message", (message,)) {
                        error!("[{}] `on_message` has failed: {}", service_id, error);
                    }
                    (engine, ast, scope)
                })
                .await;
                engine = engine_;
                ast = ast_;
                scope = scope_;
            }

            unreachable!();
        });

        Ok(())
    }

    fn register_global_functions(service_id: &str, engine: &mut Engine) {
        Self::register_logging_functions(service_id, engine);
        Self::register_standard_functions(engine);
    }

    fn register_functions(engine: &mut Engine, tx: Sender) {
        Self::register_debug_functions::<MessageType>(engine);
        Self::register_debug_functions::<DateTime<Local>>(engine);

        Self::register_message_functions(engine, tx);
        Self::register_value_functions(engine);

        telegram::register_functions(engine);
    }

    fn push_constants(scope: &mut Scope) {
        scope.push_constant("message_read_non_logged", MessageType::ReadNonLogged);
        scope.push_constant("message_read_logged", MessageType::ReadLogged);
        scope.push_constant("message_write", MessageType::Write);
    }

    /// Assigns the service instances to the inner variables.
    fn push_services(scope: &mut Scope, services: HashMap<String, Service>) {
        for (service_id, service) in services.into_iter() {
            #[allow(clippy::single_match)]
            match service {
                Service::Telegram(telegram) => {
                    scope.push_constant(service_id, telegram);
                }
                _ => (),
            }
        }
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

    fn register_debug_functions<T: std::fmt::Debug + Clone + Send + Sync + 'static>(engine: &mut Engine) {
        engine.register_fn("to_string", to_debug_string::<T>);
        engine.register_fn("print", to_debug_string::<T>);
        engine.register_fn("debug", to_debug_string::<T>);
        engine.register_fn("+", |left: &str, right: T| left.to_owned() + &format!("{:?}", right));
        engine.register_fn("+", |left: T, right: &str| format!("{:?}", left) + right);
    }

    /// Registers `Message` functions.
    fn register_message_functions(engine: &mut Engine, tx: Sender) {
        Self::register_debug_functions::<Message>(engine);

        engine.register_fn("new_message", Message::new::<String>);
        engine.register_fn("send", move |this: &mut Message| {
            let this = this.clone();
            let mut tx = tx.clone();
            task::spawn(async move { this.send_to(&mut tx).await });
        });

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
        engine.register_get_set(
            "sensor_title",
            |this: &mut Message| this.sensor.title.clone(),
            |this: &mut Message, title: Option<String>| {
                this.sensor.title = title;
            },
        );
        engine.register_get_set(
            "timestamp",
            |this: &mut Message| this.reading.timestamp,
            |this: &mut Message, timestamp: DateTime<Local>| {
                this.reading.timestamp = timestamp;
            },
        );
    }

    fn register_value_functions(engine: &mut Engine) {
        Self::register_debug_functions::<Value>(engine);

        engine.register_get("inner", |this: &mut Value| -> Dynamic {
            let this: &Value = this;
            if let Ok(value) = this.try_into() {
                Dynamic::from::<f64>(value)
            } else if let Ok(value) = this.try_into() {
                Dynamic::from::<i64>(value)
            } else if let Ok(value) = this.try_into() {
                Dynamic::from::<bool>(value)
            } else if let Ok(value) = this.try_into() {
                Dynamic::from::<String>(value)
            } else if let Ok(value) = this.try_into() {
                Dynamic::from::<Arc<Bytes>>(value)
            } else {
                Dynamic::from(())
            }
        });
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
        .map_err(|error| error.to_string())?;
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

    #[test]
    fn test_value_inner_ok() -> Result {
        let mut engine = Engine::new();
        let mut scope = Scope::new();
        Rhai::register_value_functions(&mut engine);

        scope.push("value", Value::Counter(42));
        assert_eq!(engine.eval_with_scope::<i64>(&mut scope, "value.inner")?, 42);

        scope.push("value", Value::Text("hello".into()));
        assert_eq!(engine.eval_with_scope::<String>(&mut scope, "value.inner")?, "hello");

        scope.push("value", Value::Boolean(true));
        assert_eq!(engine.eval_with_scope::<bool>(&mut scope, "value.inner")?, true);

        scope.push("value", Value::None);
        assert_eq!(engine.eval_with_scope::<()>(&mut scope, "value.inner")?, ());

        scope.push("value", Value::Temperature(42.0));
        assert_eq!(engine.eval_with_scope::<f64>(&mut scope, "value.inner")?, 42.0);

        Ok(())
    }
}
