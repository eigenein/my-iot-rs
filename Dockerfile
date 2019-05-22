# My IoT for Raspberry Pi Zero (W)
FROM rust:1.34 AS build

# Set up cross compilation to ARMv6.
RUN \
    rustup target add arm-unknown-linux-gnueabihf \
    && git clone --depth=1 https://github.com/raspberrypi/tools.git /tmp/tools
ENV \
    CARGO_TARGET_ARM_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc \
    PATH=/tmp/tools/arm-bcm2708/arm-linux-gnueabihf/bin:${PATH}

# Not using the dirty dependency caching hack, because it's not stable.
RUN USER=root cargo new my-iot
WORKDIR /my-iot
COPY . .
RUN cargo build --release --target arm-unknown-linux-gnueabihf

# Make the final tiny image for Raspberry Pi Zero (W).
FROM balenalib/rpi-debian:run
RUN install_packages libcap2-bin
COPY --from=build /my-iot/target/arm-unknown-linux-gnueabihf/release/my-iot /usr/local/bin/my-iot
RUN setcap cap_net_raw=ep /usr/local/bin/my-iot
USER nobody:nogroup
WORKDIR /app
EXPOSE 8080
ENV RUST_LOG=my_iot=info
CMD ["my-iot"]
