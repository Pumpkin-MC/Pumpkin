FROM rust:1-alpine3.21 AS builder
ARG GIT_VERSION=Docker
ENV GIT_VERSION=$GIT_VERSION
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN apk add --no-cache musl-dev

WORKDIR /pumpkin
COPY . /pumpkin

RUN rustup show active-toolchain || rustup toolchain install

# build release
RUN --mount=type=cache,sharing=private,target=/pumpkin/target \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --release && cp target/release/pumpkin ./pumpkin.release

# strip debug symbols from binary
RUN strip pumpkin.release

FROM alpine:3.21

# Identifying information for registries like ghcr.io
LABEL org.opencontainers.image.source=https://github.com/Pumpkin-MC/Pumpkin

RUN apk add --no-cache libgcc \
    && adduser --disabled-password --home /home/container container

WORKDIR /home/container

COPY --from=builder /pumpkin/pumpkin.release /bin/pumpkin

USER container
ENV USER=container HOME=/home/container

CMD [ "/bin/pumpkin" ]
