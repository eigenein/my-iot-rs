Yet another [home automation](https://en.wikipedia.org/wiki/Home_automation). Written in [Rust](https://www.rust-lang.org/).

At the moment all documentation resides in the `README`. It is incomplete and will likely stay incomplete until features will be stabilized. Nevertheless, I strive to maintain it.

[![Wiki](https://img.shields.io/badge/-Wiki-orange.svg?logo=github)](https://github.com/eigenein/my-iot-rs/wiki)
[![rustdoc](https://img.shields.io/badge/-rustdoc-lightgray.svg?logo=rust)](https://eigenein.github.io/my-iot-rs/)
[![Crates.io](https://img.shields.io/crates/v/my-iot?logo=rust)](https://crates.io/crates/my-iot)
[![Crates.io](https://img.shields.io/crates/l/my-iot)](https://crates.io/crates/my-iot)
[![Build Status](https://github.com/eigenein/my-iot-rs/workflows/build/badge.svg)](https://github.com/eigenein/my-iot-rs/actions)
[![GitHub last commit](https://img.shields.io/github/last-commit/eigenein/my-iot-rs?logo=github)](https://github.com/eigenein/my-iot-rs/commits/master)

- [What Is It](#what-is-it)
- [Installation](#installation)
- [Services](#services)
- [Run at System Startup](#run-at-system-startup)
- [Publish on the Internet with NGINX](#publish-on-the-internet-with-nginx)

# What Is It?

I'm writing something similar to [Home Assistant](https://www.home-assistant.io/) from ground up for my tiny set of Wi-Fi enabled devices.

And no, I didn't think about the project name long enough.

## Why?

- I want to learn [Rust](https://www.rust-lang.org/)
- I want it to be as less configurable as possible
- I want it to run fast on my [Raspberry Pi Zero W](https://www.raspberrypi.org/products/raspberry-pi-zero-w/)

## Stack

- [Rust](https://www.rust-lang.org/)
- [Rouille](https://github.com/tomaka/rouille)
- [Askama](https://github.com/djc/askama)
- [Bulma](https://bulma.io/)
- [Lua](https://www.lua.org/)

# Installation

There're few different ways to install My IoT. Either way, you get a single executable `my-iot`. Nothing else is needed.

## Pre-compiled binaries for Raspberry Pi Zero W

```bash
wget `curl -s https://api.github.com/repos/eigenein/my-iot-rs/releases/latest | grep -o "https://github.com/eigenein/my-iot-rs/releases/download/.*/my-iot-arm-unknown-linux-gnueabihf"` -O ~/bin/my-iot.new && chmod +x ~/bin/my-iot.new && sudo setcap cap_net_raw+ep ~/bin/my-iot.new && mv ~/bin/my-iot.new ~/bin/my-iot && sudo service my-iot restart
```

## Install from crates.io

```bash
cargo install my-iot
```

## Compile from sources

```bash
git clone https://github.com/eigenein/my-iot-rs.git
cd my-iot-rs
make
sudo make install
```

## Cross-compile for Raspberry Pi Zero W

```bash
make docker/build/arm-unknown-linux-gnueabihf
```

### File capabilities

You may need to manually set capabilities on the produced binary:

```bash
setcap cap_net_raw+ep /home/pi/.cargo/bin/my-iot
```

This is needed to use some low-level protocols (for instance, [ICMP](https://en.wikipedia.org/wiki/Internet_Control_Message_Protocol) for the [[Ping Service]]) as a non-root user.

# Settings

My IoT is configured with a single [TOML](https://github.com/toml-lang/toml) file. By default, `my-iot.toml` is read from `~/.config` directory.

## Example

```toml
http_port = 8080

[services.heartbeat]
type = "Clock"
interval_ms = 2000

[services.buienradar]
type = "Buienradar"
station_id = 6240
```

# Services

**Service** is a kind of interface between My IoT and the real world. You can set up as many services as you want, even multiple services of a same type. A service is typically capable of:
- Producing messages about something is happening
- Listening to other services messages and reacting on them

## [Lua](https://www.lua.org/)

**Lua** service allows to react on incoming messages via a Lua script, so you can implement virtually anything.

### Example

```toml
[services.lua]
type = "Lua"
script = '''
    function on_message(message)
      info(message.sensor_id)
      info(message.type)
    end
'''
```

### Builtins

My IoT adds some extra globals to execution context.

#### `debug`, `info`, `warn`, `error`

These functions are similar to the Rust's ones, but they only accept a single string literal as the only parameter. Whatever you pass there will go through My IoT logging and so is manageable by e.g. `journalctl` or whatever logging system you use.

# Run at System Startup

For now please refer to [Raspberry Pi systemd page](https://www.raspberrypi.org/documentation/linux/usage/systemd.md).

## Example

```bash
cat /lib/systemd/system/my-iot.service
```

```ini
[Unit]
Description = my-iot
After = network.target

[Service]
ExecStart = /home/pi/.cargo/bin/my-iot --silent
WorkingDirectory = /home/pi
StandardOutput = journal
StandardError = journal
Restart = always
User = pi

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl enable my-iot
sudo systemctl status my-iot
sudo systemctl start my-iot
sudo systemctl stop my-iot
sudo systemctl restart my-iot
```

## Logs

```sh
journalctl -f -u my-iot
```

See also: [How To Use Journalctl to View and Manipulate Systemd Logs](https://www.digitalocean.com/community/tutorials/how-to-use-journalctl-to-view-and-manipulate-systemd-logs).

# [Publish on the Internet with NGINX](https://docs.nginx.com/nginx/admin-guide/web-server/reverse-proxy/)

## Checklist

- Configure [Let's Encrypt](https://letsencrypt.org/) or another certificate provider
- Set right certificate and private key paths
- Generate `.htpasswd` or configure another way of authentication

## Example

```nginx
events { }

http {
    upstream backend {
        server 127.0.0.1:8081;
        keepalive 32;
    }

    server {
        listen 443 ssl default_server;
        listen [::]:443 default_server;
        charset utf-8;

        add_header Strict-Transport-Security max-age=2592000;

        ssl_session_cache shared:SSL:10m;
        ssl_session_timeout 10m;
        ssl_protocols  TLSv1 TLSv1.1 TLSv1.2;
        ssl_certificate /etc/letsencrypt/live/example.com/cert.pem;
        ssl_certificate_key /etc/letsencrypt/live/example.com/privkey.pem;
        ssl_ciphers 'ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES256-GCM-SHA384:ECDHE-ECDSA-AES256-GCM-SHA384:DHE-RSA-AES128-GCM-SHA256:DHE-DSS-AES128-GCM-SHA256:ECDHE-RSA-AES128-SHA256:ECDHE-ECDSA-AES128-SHA256:ECDHE-RSA-AES128-SHA:ECDHE-ECDSA-AES128-SHA:ECDHE-RSA-AES256-SHA384:ECDHE-ECDSA-AES256-SHA384:ECDHE-RSA-AES256-SHA:ECDHE-ECDSA-AES256-SHA:DHE-RSA-AES128-SHA256:DHE-RSA-AES128-SHA:DHE-DSS-AES128-SHA256:DHE-RSA-AES256-SHA256:DHE-DSS-AES256-SHA:DHE-RSA-AES256-SHA:AES128-GCM-SHA256:AES256-GCM-SHA384:AES128-SHA256:AES256-SHA256:AES128-SHA:AES256-SHA:AES:CAMELLIA:DES-CBC3-SHA:!aNULL:!eNULL:!EXPORT:!DES:!RC4:!MD5:!PSK:!aECDH:!EDH-DSS-DES-CBC3-SHA:!EDH-RSA-DES-CBC3-SHA:!KRB5-DES-CBC3-SHA';
        ssl_prefer_server_ciphers on;

        gzip on;
        gzip_buffers 16 8k;
        gzip_comp_level 6;
        gzip_http_version 1.1;
        gzip_min_length 256;
        gzip_proxied any;
        gzip_vary on;
        gzip_types
            text/xml application/xml application/atom+xml application/rss+xml application/xhtml+xml image/svg+xml
            text/javascript application/javascript application/x-javascript
            text/x-json application/json application/x-web-app-manifest+json
            text/css text/plain text/x-component
            font/opentype application/x-font-ttf application/vnd.ms-fontobject
            image/x-icon;
        gzip_disable "msie6";

        auth_basic "My IoT";
        auth_basic_user_file /etc/.htpasswd;

        location / {
            proxy_pass http://backend;
            proxy_http_version 1.1;
            proxy_set_header Connection "";
            proxy_set_header Host $http_host;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Real-IP $remote_addr;
        }
    }
}
```
