FROM rust:1.39 AS base
ENV OPENSSL_STATIC=1

# Raspberry Pi Zero (W)
FROM base AS arm-unknown-linux-gnueabihf

# Install build tools.
RUN rustup target add arm-unknown-linux-gnueabihf
RUN git clone --depth=1 https://github.com/raspberrypi/tools.git
ENV CARGO_TARGET_ARM_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc

# Build OpenSSL.
RUN curl "https://www.openssl.org/source/openssl-1.0.2t.tar.gz" | tar zxf -
ENV \
    PATH=$PWD/tools/arm-bcm2708/arm-linux-gnueabihf/bin:$PATH \
    PREFIX=/usr/local \
    MACHINE=armv6 \
    ARCH=arm \
    CC=arm-linux-gnueabihf-gcc \
    AR=arm-linux-gnueabihf-ar \
    RANLIB=arm-linux-gnueabihf-ranlib
RUN cd openssl-1.0.2t && ./config shared --prefix=$PREFIX && make && make install
ENV \
    OPENSSL_LIB_DIR=/usr/local/lib \
    OPENSSL_INCLUDE_DIR=/usr/local/include

ENTRYPOINT cargo build --release --target arm-unknown-linux-gnueabihf --manifest-path /my-iot-rs/Cargo.toml
