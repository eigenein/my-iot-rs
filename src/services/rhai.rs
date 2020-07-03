use crate::prelude::*;
use crate::settings::Service;
use rhai::{Array, Dynamic, Engine, EvalAltResult, RegisterFn, RegisterResultFn, Scope};
use std::process::Command;

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Rhai {
    script: String,
}

impl Rhai {
    pub fn spawn(self, service_id: String, bus: &mut Bus, _services: HashMap<String, Service>) -> Result<()> {
        let tx = bus.add_tx();
        let rx = bus.add_rx();

        info!("[{}] Compiling the script…", service_id);
        let mut engine = Engine::new();
        let ast = engine.compile(&self.script)?;
        let mut scope = Scope::new();

        Self::register_types(&mut engine);
        Self::register_values(&mut scope);
        Self::register_global_functions(&mut engine, tx);

        // Sand-boxing.
        let engine = engine;

        info!("[{}] Executing the script…", service_id);
        engine.consume_ast_with_scope(&mut scope, &ast)?;

        thread::Builder::new().name(service_id.clone()).spawn(move || {
            for message in &rx {
                if let Err(error) = engine.call_fn::<_, ()>(&mut scope, &ast, "on_message", (message,)) {
                    error!("[{}] `on_message` has failed: {}", &service_id, error.to_string());
                }
            }

            unreachable!();
        })?;

        Ok(())
    }

    fn register_types(engine: &mut Engine) {
        Self::register_message_type(engine);
        Self::register_message(engine);
    }

    fn register_values(scope: &mut Scope) {
        scope.push_constant("message_read_non_logged", MessageType::ReadNonLogged);
        scope.push_constant("message_read_logged", MessageType::ReadLogged);
        scope.push_constant("message_write", MessageType::Write);
    }

    fn register_message_type(engine: &mut Engine) {
        engine.register_type::<MessageType>();
        engine.register_fn("to_string", get_string::<MessageType>);
        engine.register_fn("print", get_string::<MessageType>);
        engine.register_fn("debug", get_string::<MessageType>);
    }

    fn register_message(engine: &mut Engine) {
        engine.register_type::<Message>();
        engine.register_fn("new_message", Message::new::<String>);
        engine.register_fn("to_string", get_string::<Message>);
        engine.register_fn("print", get_string::<Message>);
        engine.register_fn("debug", get_string::<Message>);
        engine.register_get_set(
            "sensor_id",
            |message: &mut Message| message.sensor.id.clone(),
            |message: &mut Message, sensor_id: String| {
                message.sensor.id = sensor_id;
            },
        );
        engine.register_get_set(
            "type",
            |message: &mut Message| message.type_,
            |message: &mut Message, type_: MessageType| {
                message.type_ = type_;
            },
        );
        // TODO: more getters and setters.
    }

    fn register_global_functions(engine: &mut Engine, tx: Sender) {
        Self::register_clean_global_functions(engine);

        engine.register_fn("send_message", move |message: Message| {
            message.send_and_forget(&tx);
        });
    }

    fn register_clean_global_functions(engine: &mut Engine) {
        engine.on_print(|string| info!("{}", string));
        engine.on_debug(|string| debug!("{}", string));
        engine.register_result_fn("spawn_process", spawn_process);
    }
}

/// Used to spawn an external process.
fn spawn_process(program: &str, args: Array) -> Result<Dynamic, Box<EvalAltResult>> {
    Command::new(program)
        .args(args.iter().map(|arg| arg.to_string()))
        .spawn()
        .map_err(|error| error.to_string())?;
    Ok(().into())
}

fn get_string<T: std::fmt::Debug>(this: &mut T) -> String {
    format!("{:?}", this)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Result;

    #[test]
    fn spawn_process_ok() -> Result<()> {
        let mut engine = Engine::new();
        Rhai::register_clean_global_functions(&mut engine);
        engine.eval(r#"spawn_process("echo", ["-n"])"#)?;
        Ok(())
    }
}
