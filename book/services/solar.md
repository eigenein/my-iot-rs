# Solar

Provides sensors with durations to and after sunrise and sunset:
- `{service_id}::before::sunrise`
- `{service_id}::after::sunrise`
- `{service_id}::before::sunset`
- `{service_id}::after::sunset`

For polar night and day the following non-logged messages get emitted:
- `{service_id}::polar::day`
- `{service_id}::polar::night`

## Settings

```toml
[services.my_solar]
type = "Solar"

# Room title used for the sensors.
room_title = "Home"

# Refresh interval.
interval_millis = 60000

# Sensor reading expiration time.
ttl_millis = 120000

[services.my_solar.secrets]
latitude = 12.345678
longitude = 12.345678
```
