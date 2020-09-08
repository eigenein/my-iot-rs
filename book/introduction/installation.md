# Installation

There are a few different ways to install My IoT. Either way, you get a single executable `my-iot`:

- Grab a prebuilt binary from the GitHub releases: https://github.com/eigenein/my-iot-rs/releases
- Install from crates.io: `cargo install my-iot`
- Cross-compile: `cargo install cross && cross build --target …`

**File Capabilities**

You may need to manually set the file capabilities on the produced binary:

```bash
setcap cap_net_raw+ep /home/pi/.cargo/bin/my-iot
```

This is needed to use some low-level protocols (for instance, [ICMP]) as a non-root user.

[ICMP]: https://en.wikipedia.org/wiki/Internet_Control_Message_Protocol
