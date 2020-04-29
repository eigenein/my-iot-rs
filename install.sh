#!/bin/bash -e
wget `curl -s https://api.github.com/repos/eigenein/my-iot-rs/releases/latest | grep -o "https://github.com/eigenein/my-iot-rs/releases/download/.*/my-iot-arm-unknown-linux-gnueabihf"` -O ~/bin/my-iot.new
chmod +x ~/bin/my-iot.new
sudo setcap cap_net_raw+ep ~/bin/my-iot.new
mv ~/bin/my-iot.new ~/bin/my-iot
sudo service my-iot restart
