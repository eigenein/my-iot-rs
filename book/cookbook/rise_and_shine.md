# «Rise and shine» IKEA Trådfri lights starting one hour before sunset

At the moment of writing the recipe there is no native `Tradfri` service. I'm following [the `coap-client` tutorial](https://github.com/glenndehaan/ikea-tradfri-coap-docs/blob/master/README.md) to control bulbs.

```toml
[services.sun_vijfhuizen]
type = "Solar"
latitude = 52.000000
longitude = 4.000000
room_title = "Vijfhuizen"

[services.rise_and_shine]
type = "Rhai"
script = '''
    fn on_message(message) {
        if message.sensor_id == "sun_vijfhuizen::before::sunset" && message.value.inner < 3600.0 {
            let brightness = 255 * (3600 - message.value.inner.to_int()) / 3600;
            print("Brightness: " + brightness);
            spawn_process("coap-client", [
                "-m",
                "put",
                "-u",
                "eigenein",
                "-k",
                "2GOjFumz6iVnecdt",
                "-e",
                "{\"5851\": " + brightness + "}",
                "coaps://GW-A0C9A0679CBB.home:5684/15004/131080",
            ]);
        }
    }
'''
