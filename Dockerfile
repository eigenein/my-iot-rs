FROM rustlang/rust:nightly-buster-slim AS base
LABEL maintainer="Pavel Perestoronin <eigenein@gmail.com>"
LABEL org.label-schema.description="My IoT builder for different devices"
LABEL org.label-schema.vcs-url="https://github.com/eigenein/my-iot-rs"

# Raspberry base
# --------------------------------------------------------------------------------------------------

FROM base AS base-raspberry

RUN apt-get update && apt-get install --assume-yes make

ENV \
    ARCH=arm \
    AR=arm-linux-gnueabihf-ar \
    CC=arm-linux-gnueabihf-gcc \
    RANLIB=arm-linux-gnueabihf-ranlib

# Raspberry Pi 0/1
# --------------------------------------------------------------------------------------------------

FROM base-raspberry AS arm-unknown-linux-gnueabihf

RUN apt-get install --assume-yes git

# The build tools are required for this architecture.
RUN git clone --depth=1 https://github.com/raspberrypi/tools.git

ENV \
    PATH=$PWD/tools/arm-bcm2708/arm-linux-gnueabihf/bin:$PATH \
    MACHINE=armv6 \
    CARGO_TARGET_ARM_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc

RUN rustup target add arm-unknown-linux-gnueabihf
ENTRYPOINT cargo build --release --target arm-unknown-linux-gnueabihf --manifest-path /my-iot-rs/Cargo.toml

# Raspberry Pi 2/3/4
# --------------------------------------------------------------------------------------------------

FROM base-raspberry AS armv7-unknown-linux-gnueabihf

RUN apt-get install --assume-yes gcc-arm-linux-gnueabihf

ENV \
    MACHINE=armv7 \
    CARGO_BUILD_TARGET=armv7-unkown-linux-gnueabihf \
    CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc

RUN rustup target add armv7-unknown-linux-gnueabihf
ENTRYPOINT cargo build --release --target armv7-unknown-linux-gnueabihf --manifest-path /my-iot-rs/Cargo.toml
