FROM rust:1.39 AS base
ENV OPENSSL_STATIC=1

FROM base AS arm-unknown-linux-gnueabihf
RUN rustup target add arm-unknown-linux-gnueabihf
RUN git clone --depth=1 https://github.com/raspberrypi/tools.git
ENV \
    CARGO_TARGET_ARM_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc \
    PATH=$PWD/tools/arm-bcm2708/arm-linux-gnueabihf/bin:${PATH}
# TODO: OpenSSL for gnueabihf.

ENTRYPOINT cargo build --release --target arm-unknown-linux-gnueabihf --manifest-path /my-iot-rs/Cargo.toml
