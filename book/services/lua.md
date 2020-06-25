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

## Recipes

I store my settings on [GitHub Gist](https://gist.github.com/eigenein/f4af6ca1d8db0cd36de981429aa3e6e2) as an extra example.

### «Rise and shine» IKEA Trådfri lights starting one hour before sunset

At the moment of writing the recipe there is no native `Tradfri` service. I'm following [the `coap-client` tutorial](https://github.com/glenndehaan/ikea-tradfri-coap-docs/blob/master/README.md) to control bulbs.

```toml
[services.sun_vijfhuizen]
type = "Solar"
latitude = 52.000000
longitude = 4.000000
room_title = "Vijfhuizen"

[services.rise_and_shine]
type = "Lua"
filter_sensor_ids = "^sun_vijfhuizen::before::sunset$"
script = '''
function onMessage(message)
  if message.value < 3600 then
    os.execute(string.format(
      "coap-client -m put -u user -k shared_key -e '{\"5851\": %d}' coaps://GW-XXXXXXXXXXXX.home:5684/15004/131080",
      math.floor(255 * ((3600 - message.value) / 3600))
    ))
  end
end
```
