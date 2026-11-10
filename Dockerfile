FROM ubuntu:26.04

ARG TARGETARCH
ARG TARGETVARIANT
ARG PUMPKIN_TAG=nightly
ARG USE_X64_V3=auto

RUN apt-get -o Acquire::Retries=3 update && \
    DEBIAN_FRONTEND=noninteractive apt-get -o Acquire::Retries=3 install -y --no-install-recommends curl ca-certificates netcat-openbsd && \
    case "${TARGETARCH}" in \
        "amd64") \
            if [ "${TARGETVARIANT}" = "v3" ] || [ "${USE_X64_V3}" = "true" ]; then \
                BIN_ARCH="X64-v3"; \
            elif [ "${USE_X64_V3}" = "auto" ] && /lib64/ld-linux-x86-64.so.2 --help 2>/dev/null | grep -q "x86-64-v3 (supported"; then \
                BIN_ARCH="X64-v3"; \
            else \
                BIN_ARCH="X64"; \
            fi \
            ;; \
        "arm64") BIN_ARCH="ARM64" ;; \
        *) echo "Unsupported architecture: ${TARGETARCH}" && exit 1 ;; \
    esac && \
    if [ "${PUMPKIN_TAG}" = "latest" ]; then \
        DOWNLOAD_URL="https://github.com/Pumpkin-MC/Pumpkin/releases/latest/download/pumpkin-${BIN_ARCH}-Linux"; \
    else \
        DOWNLOAD_URL="https://github.com/Pumpkin-MC/Pumpkin/releases/download/${PUMPKIN_TAG}/pumpkin-${BIN_ARCH}-Linux"; \
    fi && \
    curl -fsSL --retry 3 "${DOWNLOAD_URL}" -o /usr/local/bin/pumpkin && \
    chmod +x /usr/local/bin/pumpkin && \
    apt-get purge -y --auto-remove curl && \
    rm -rf /var/lib/apt/lists/* && \
    groupadd -g 2613 pumpkin && \
    useradd -u 2613 -g pumpkin -m -d /pumpkin pumpkin && \
    chown -R pumpkin:pumpkin /pumpkin

WORKDIR /pumpkin
USER pumpkin:pumpkin

ENV RUST_BACKTRACE=1
EXPOSE 25565 19132 19132/udp

ENTRYPOINT [ "pumpkin" ]

HEALTHCHECK --interval=30s --timeout=3s --retries=3 \
    CMD nc -z 127.0.0.1 25565 || exit 1
