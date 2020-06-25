# Settings

My IoT is configured using [TOML](https://github.com/toml-lang/toml) files specified as command-line arguments:

```bash
my-iot my-iot.toml
```

## Example

```toml
# my-iot.toml

http_port = 8080

# `heartbeat` is a user-defined service ID.
[services.heartbeat]
type = "Clock"
interval_millis = 2000

[services.weather]
type = "Buienradar"
station_id = 6240
```

## Securing Secrets

It's a common pattern to split configuration into non-secret and secret parts, where non-secret part is stored under a version control.

`my-iot` allows specifying multiple settings files, it means that you can put your secrets in a separate file excluded by `.gitignore`. Services provide separate configuration section to allow moving it out of public part.

For example:

```toml
# my-iot.toml:
[services.telegram]
type = "Telegram"

[services.sun_amsterdam]
type = "Solar"
room_title = "Amsterdam"

# secrets.toml:
[services.telegram.secrets]
token = "..."

[services.sun_amsterdam.secrets]
latitude = 52.3667
longitude = 4.8945
```

Then you run My IoT as `my-iot my-iot.toml secrets.toml`.
