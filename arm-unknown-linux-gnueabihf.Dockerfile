FROM rustembedded/cross:arm-unknown-linux-gnueabihf

LABEL maintainer = "Pavel Perestoronin <eigenein@gmail.com>"
LABEL org.label-schema.version = "1.0.0"
LABEL org.label-schema.name = "docker.pkg.github.com/eigenein/my-iot-rs/cross-arm-unknown-linux-gnueabihf"
LABEL org.label-schema.vcs-url = "https://github.com/eigenein/my-iot-rs"

ENV \
    PREFIX=/usr/local \
    OPENSSL_VERSION="1.1.1g" \
    OPENSSL_LIB_DIR=/usr/local/lib \
    OPENSSL_INCLUDE_DIR=/usr/local/include \
    AR=arm-linux-gnueabihf-ar \
    CC=arm-linux-gnueabihf-gcc
RUN curl "https://www.openssl.org/source/openssl-$OPENSSL_VERSION.tar.gz" | tar --strip-components=1 -xz
RUN \
    ./Configure shared --prefix=$PREFIX no-dso linux-armv4 -fPIC \
    && make \
    && make install
