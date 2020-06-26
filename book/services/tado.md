# tado°

[tado°](https://www.tado.com/) Smart Thermostat service, periodically polls the API.

## [Open Window Detection Skill](https://support.tado.com/en/articles/3387308-how-does-the-open-window-detection-skill-work)

This service can emulate the [Open Window Detection Skill](https://support.tado.com/en/articles/3387308-how-does-the-open-window-detection-skill-work) with the `enable_open_window_detection_skill = true` setting.

As soon as open window is detected, the service automatically activates the open window mode and emits `{service_id}::{home_id}::{zone_id}::open_window_activated` message.

## Settings

```toml
[services.my_tado]
type = "Tado"

# Enables the Open Window Detection Skill emulation.
enable_open_window_detection_skill = true

[services.my_tado.secrets]

# Account email.
email = "user@example.com"

# Account password.
password = "example"
```
