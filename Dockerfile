FROM rust:1-alpine3.21 AS builder
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN apk add --no-cache musl-dev \
    # Required for git-version
    git

WORKDIR /pumpkin
COPY . /pumpkin

RUN rustup show active-toolchain || rustup toolchain install

# build release
RUN --mount=type=cache,sharing=private,target=/pumpkin/target \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --release && cp target/release/pumpkin ./pumpkin.release

FROM alpine:3.22

COPY --from=builder /pumpkin/pumpkin.release /bin/pumpkin

# set workdir to /pumpkin, this is required to influence the PWD environment variable
# it allows for bind mounting the server files without overwriting the pumpkin
# executable (without requiring an `docker cp`-ing the binary to the host folder)
WORKDIR /pumpkin

RUN apk add --no-cache libgcc && chown 2613:2613 .

ENV RUST_BACKTRACE=1
EXPOSE 25565
USER 2613:2613
ENTRYPOINT [ "/bin/pumpkin" ]
HEALTHCHECK CMD nc -z 127.0.0.1 25565 || exit 1
