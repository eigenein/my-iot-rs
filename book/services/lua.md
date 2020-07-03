# [Lua](https://www.lua.org/)

**Lua** service allows to react on incoming messages via a Lua script, so you can implement virtually anything.

## Example

```toml
[services.example]
type = "Lua"
# language=lua
script = '''
    sendMessage("hello::wind", "READ_NON_LOGGED", {bft = 5})

    function onMessage(message)
      info(string.format("%s: %s", message.sensor_id, message.value))
    end
'''
```

## `onMessage`

User function with the name of `onMessage` gets called on each message and receives one argument `message` which is a table that contains the following indices:

- `sensor_id`
- `type`
- `room_title`
- `sensor_title`
- `value`
- `timestamp_millis`

## Builtins

My IoT adds some extra globals to execution context:

### `debug`, `info`, `warn` and `error`

These functions are similar to the Rust's ones, but they only accept a single string literal as the only parameter. Whatever you pass there will go through My IoT logging and so is manageable by e.g. `journalctl` or whatever logging system you use.

### `function sendMessage(sensor_id, type, {args})`

Sends out a message with the given `sensor_id` (string), `type` (see below) and arguments `args` (see below). You use this to control other services as well as provide custom sensors.

Possible message `type`-s are:

| Constant            |      |
|---------------------|------|
| `"READ_LOGGED"`     | TODO |
| `"READ_NON_LOGGED"` | TODO |
| `"READ_SNAPSHOT"`   | TODO |
| `"WRITE"`           | TODO |

Optional parameter `args` is a table, which may provide additional message details:

| Index                 | Type    |      |
|-----------------------|---------|------|
| `room_title`          | string  | TODO |
| `sensor_title`        | string  | TODO |
| `timestamp_millis`    | number  | TODO |

All indices are optional. Also, a value is provided via either of the following indices in `args`:

| Index            | Type    |                                                       |
|------------------|---------|-------------------------------------------------------|
| `bft`            | integer | Beaufort wind force                                   |
| `counter`        | integer | Unsigned unit-less counter                            |
| `image_url`      | string  | Image URL                                             |
| `bool`           | boolean | `true` and `false`                                    |
| `wind_direction` | string  | Point of the compass that represents a wind direction |
| `data_size`      | integer | Data size in bytes                                    |
| `text`           | string  | Plain text                                            |
| `rh`             | number  | Relative humidity in percents                         |
| `celsius`        | number  | Temperature in degrees Celsius                        |
| `kelvin`         | number  | Temperature in Kelvins                                |
| `meters`         | number  | Length in meters                                      |

#### [Points of the Compass](https://en.wikipedia.org/wiki/Points_of_the_compass)

TODO
