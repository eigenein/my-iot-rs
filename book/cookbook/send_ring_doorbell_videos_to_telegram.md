# Send Ring doorbell videos to Telegram

```toml
[services.notify_ring]
type = "Rhai"
script = '''
    const chat_id = …;

    fn on_message(message) {
        if message.sensor_id.starts_with("ring::doorbot::32333947::recording::") {
            telegram.send_video(chat_id, message.value.inner.bytes);
        }
    }
'''
```
