# «Rise and shine» IKEA Trådfri lights starting one hour before sunset

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
