FROM ubuntu:26.04

ARG TARGETARCH
ARG PUMPKIN_TAG=nightly

RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends curl ca-certificates netcat-openbsd && \
    case "${TARGETARCH}" in \
        "amd64") BIN_ARCH="X64" ;; \
        "arm64") BIN_ARCH="ARM64" ;; \
        *) echo "Unsupported architecture: ${TARGETARCH}" && exit 1 ;; \
    esac && \
    curl -fsSL "https://github.com/Pumpkin-MC/Pumpkin/releases/download/${PUMPKIN_TAG}/pumpkin-${BIN_ARCH}-Linux" \
        -o /usr/local/bin/pumpkin && \
    chmod +x /usr/local/bin/pumpkin && \
    apt-get purge -y --auto-remove curl && \
    rm -rf /var/lib/apt/lists/*

RUN groupadd -g 2613 pumpkin && \
    useradd -u 2613 -g pumpkin -m -d /pumpkin pumpkin && \
    chown -R pumpkin:pumpkin /pumpkin

WORKDIR /pumpkin
USER pumpkin:pumpkin

ENV RUST_BACKTRACE=1
EXPOSE 25565

ENTRYPOINT [ "pumpkin" ]

HEALTHCHECK --interval=30s --timeout=3s --retries=3 \
    CMD nc -z 127.0.0.1 25565 || exit 1
