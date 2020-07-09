# [Telegram](https://core.telegram.org/bots/api)

Implements a [Telegram Bot](https://core.telegram.org/bots).

## Settings

You'll need an [authorization token](https://core.telegram.org/bots#creating-a-new-bot):

```toml
[services.service_id]
type = "Telegram"

[services.service_id.secrets]
token = "123456789:FoObAr"
```
