# Notify tado° open window

```toml
[services.notify_open_window]
type = "Rhai"
script = '''
    fn on_message(message) {
        if message.sensor_id == "tado::469375::1::open_window_activated" {
            telegram.send_message(
                -1001349838037,
                "💨 Открыто окно в *" + message.location + "* @eigenein",
                #{parse_mode: "MarkdownV2"},
            );
        }
    }
'''
```
