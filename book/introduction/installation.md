# Installation

There're few different ways to install My IoT. Either way, you get a single executable `my-iot`.

---

**File Capabilities**

You may need to manually set capabilities on the produced binary:

```bash
setcap cap_net_raw+ep /home/pi/.cargo/bin/my-iot
```

This is needed to use some low-level protocols (for instance, [ICMP](https://en.wikipedia.org/wiki/Internet_Control_Message_Protocol)) as a non-root user.

---

## Pre-compiled binaries for Raspberry Pi Zero W

```bash
curl -fsSL https://raw.githubusercontent.com/eigenein/my-iot-rs/master/install.sh | bash
```

## Install from crates.io

```bash
cargo install my-iot
```

## Cross-compile for Raspberry Pi Zero W

```bash
make docker/build/arm-unknown-linux-gnueabihf
```
