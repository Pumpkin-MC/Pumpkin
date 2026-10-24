FROM alpine:3.24

ARG TARGETARCH
ARG PUMPKIN_TAG=nightly

RUN apk add --no-cache curl ca-certificates && \
    case "${TARGETARCH}" in \
        "amd64") BIN_ARCH="X64" ;; \
        "arm64") BIN_ARCH="ARM64" ;; \
        *) echo "Unsupported architecture: ${TARGETARCH}" && exit 1 ;; \
    esac && \
    curl -fsSL "https://github.com/Pumpkin-MC/Pumpkin/releases/download/${PUMPKIN_TAG}/pumpkin-${BIN_ARCH}-Linux-musl" \
        -o /usr/local/bin/pumpkin && \
    chmod +x /usr/local/bin/pumpkin && \
    apk del curl

RUN addgroup -g 2613 pumpkin && \
    adduser -u 2613 -G pumpkin -D -h /pumpkin pumpkin && \
    chown -R pumpkin:pumpkin /pumpkin

WORKDIR /pumpkin
USER pumpkin:pumpkin

ENV RUST_BACKTRACE=1
EXPOSE 25565

ENTRYPOINT [ "pumpkin" ]

HEALTHCHECK --interval=30s --timeout=3s --retries=3 \
    CMD nc -z 127.0.0.1 25565 || exit 1