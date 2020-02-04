FROM rust:1.41-buster AS base
LABEL maintainer="Pavel Perestoronin <eigenein@gmail.com>"
LABEL org.label-schema.description="My IoT builder for different devices"
LABEL org.label-schema.vcs-url="https://github.com/eigenein/my-iot-rs"

# Raspberry Pi Zero (W)
# --------------------------------------------------------------------------------------------------

FROM base AS arm-unknown-linux-gnueabihf

# Install build tools.
RUN rustup target add arm-unknown-linux-gnueabihf
RUN \
    git clone --depth=1 https://github.com/raspberrypi/tools.git \
    && cd tools \
    && git checkout b0c869bc929587a7e1d20a98e2dc828a24ca396a
ENV \
    PATH=$PWD/tools/arm-bcm2708/arm-linux-gnueabihf/bin:$PATH \
    PREFIX=/usr/local \
    ARCH=arm \
    MACHINE=armv6 \
    AR=arm-linux-gnueabihf-ar \
    CC=arm-linux-gnueabihf-gcc \
    RANLIB=arm-linux-gnueabihf-ranlib \
    CARGO_TARGET_ARM_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc

# Build OpenSSL.
ENV \
    OPENSSL_VERSION="1.1.1d" \
    OPENSSL_LIB_DIR=/usr/local/lib \
    OPENSSL_INCLUDE_DIR=/usr/local/include
RUN \
    curl "https://www.openssl.org/source/openssl-$OPENSSL_VERSION.tar.gz" | tar zxf - \
    && cd openssl-$OPENSSL_VERSION \
    && ./config shared --prefix=$PREFIX \
    && make \
    && make install

# Cross-build `my-iot-rs`.
ENTRYPOINT cargo build --release --target arm-unknown-linux-gnueabihf --manifest-path /my-iot-rs/Cargo.toml
