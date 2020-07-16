FROM rustlang/rust:nightly-buster-slim AS base
LABEL maintainer="Pavel Perestoronin <eigenein@gmail.com>"
LABEL org.label-schema.description="My IoT builder for different devices"
LABEL org.label-schema.vcs-url="https://github.com/eigenein/my-iot-rs"

ENV \
    PREFIX=/usr/local \
    OPENSSL_VERSION="1.1.1d" \
    OPENSSL_LIB_DIR=/usr/local/lib \
    OPENSSL_INCLUDE_DIR=/usr/local/include

RUN apt-get update && apt-get install --assume-yes curl make

# Download OpenSSL, we'll need it for the all targets.
RUN curl "https://www.openssl.org/source/openssl-$OPENSSL_VERSION.tar.gz" | tar zxf -

# Raspberry Pi 0/1
# --------------------------------------------------------------------------------------------------

FROM base AS arm-unknown-linux-gnueabihf

# Install build tools.
RUN rustup target add arm-unknown-linux-gnueabihf
RUN apt-get install --assume-yes git
RUN \
    git clone --depth=1 https://github.com/raspberrypi/tools.git \
    && cd tools \
    && git checkout 86d54c61f9a23e5b438bef98f3d1027e2c150896
ENV \
    PATH=$PWD/tools/arm-bcm2708/arm-linux-gnueabihf/bin:$PATH \
    ARCH=arm \
    MACHINE=armv6 \
    AR=arm-linux-gnueabihf-ar \
    CC=arm-linux-gnueabihf-gcc \
    RANLIB=arm-linux-gnueabihf-ranlib \
    CARGO_TARGET_ARM_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc

# Build OpenSSL.
RUN \
    cd openssl-$OPENSSL_VERSION \
    && ./config shared --prefix=$PREFIX \
    && make \
    && make install

ENTRYPOINT cargo build --release --target arm-unknown-linux-gnueabihf --manifest-path /my-iot-rs/Cargo.toml

# Raspberry Pi 2/3/4
# --------------------------------------------------------------------------------------------------

FROM base AS armv7-unknown-linux-gnueabihf

RUN rustup target add armv7-unknown-linux-gnueabihf
RUN apt-get install --assume-yes g++-arm-linux-gnueabihf libfindbin-libs-perl

ENV \
    ARCH=arm \
    MACHINE=armv7 \
    AR=arm-linux-gnueabihf-ar \
    CC=arm-linux-gnueabihf-gcc \
    RANLIB=arm-linux-gnueabihf-ranlib \
    CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc

# Build OpenSSL.
RUN \
    cd openssl-$OPENSSL_VERSION \
    && ./config shared --prefix=$PREFIX \
    && make \
    && make install

ENTRYPOINT cargo build --release --target armv7-unknown-linux-gnueabihf --manifest-path /my-iot-rs/Cargo.toml
