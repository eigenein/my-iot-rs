ARG TRIPLE
FROM rustembedded/cross:${TRIPLE}

LABEL maintainer = "Pavel Perestoronin <eigenein@gmail.com>"
LABEL org.label-schema.version = "1.0.1"
LABEL org.label-schema.name = "docker.pkg.github.com/eigenein/my-iot-rs/cross-${TRIPLE}"
LABEL org.label-schema.vcs-url = "https://github.com/eigenein/my-iot-rs"

ENV \
    PREFIX=/usr/local \
    CC=arm-linux-gnueabihf-gcc \
    AR=arm-linux-gnueabihf-ar

ENV \
    OPENSSL_VERSION="1.1.1g" \
    OPENSSL_LIB_DIR=/usr/local/lib \
    OPENSSL_INCLUDE_DIR=/usr/local/include
RUN curl "https://www.openssl.org/source/openssl-$OPENSSL_VERSION.tar.gz" | tar -xz
RUN \
    cd openssl-$OPENSSL_VERSION \
    && ./Configure shared --prefix=$PREFIX no-dso linux-armv4 -fPIC \
    && make \
    && make install_sw

# This doesn't help to build `libsqlite3-sys` crate.
RUN curl "https://www.sqlite.org/2020/sqlite-autoconf-3330000.tar.gz" | tar -xz
RUN \
    cd sqlite-autoconf-3330000 \
    && ./configure --prefix=$PREFIX \
    && make \
    && make install
