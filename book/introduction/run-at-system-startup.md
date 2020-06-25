# Run at System Startup

For now please refer to [Raspberry Pi `systemd` page](https://www.raspberrypi.org/documentation/linux/usage/systemd.md).

## Example

```bash
cat /lib/systemd/system/my-iot.service
```

```ini
[Unit]
Description = my-iot
BindsTo = network-online.target
After = network.target network-online.target

[Service]
ExecStart = /home/pi/bin/my-iot --silent my-iot/my-iot.toml my-iot/secrets.toml
WorkingDirectory = /home/pi
StandardOutput = journal
StandardError = journal
Restart = always
User = pi

[Install]
WantedBy = multi-user.target
```

```bash
sudo systemctl enable my-iot
sudo systemctl status my-iot
sudo systemctl start my-iot
sudo systemctl stop my-iot
sudo systemctl restart my-iot
```

## Logs

```bash
journalctl -u my-iot -f
```

See also: [How To Use Journalctl to View and Manipulate Systemd Logs](https://www.digitalocean.com/community/tutorials/how-to-use-journalctl-to-view-and-manipulate-systemd-logs).
